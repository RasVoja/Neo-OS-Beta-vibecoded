//! ACPI tables: what a real PC says about itself.
//!
//! QEMU puts the I/O APIC at the usual address and wires ISA interrupts
//! straight through, but a real PC says where things are in its ACPI
//! tables. This reads them once at boot:
//! - the MADT ("APIC"): how many processors, where the I/O APIC is and
//!   which GSI each ISA interrupt really arrives on (with its polarity and
//!   trigger mode);
//! - the FADT ("FACP"): the power management registers, and from the DSDT
//!   the `_S5_` sleep type, so the machine can switch itself off when Neo
//!   DOS ends instead of waiting on a halted CPU.
//!
//! The root pointer (RSDP) comes from the boot loader (PVH start info or
//! Multiboot2 tags 14/15) or, with a legacy BIOS, from the BIOS area.
//! Only the static tables are read; the AML interpreter comes much later
//! (with Genode, which has one).

use crate::arch::{inw, outb, outw};
use crate::sync::SpinLock;

/// An ISA interrupt as the I/O APIC sees it.
#[derive(Clone, Copy, Debug)]
pub struct IsaRoute {
    pub gsi: u32,
    pub active_low: bool,
    pub level: bool,
}

/// What the tables said.
#[derive(Clone, Copy, Debug)]
pub struct Acpi {
    pub revision: u8,
    pub cpus: u32,
    pub ioapic: u64,
    pub gsi_base: u32,
    pub overrides: u32,
    isa: [IsaRoute; 16],
    /// PM1a and PM1b control ports, SLP_TYP values for S5.
    pm1: (u16, u16),
    s5: Option<(u8, u8)>,
    smi: (u16, u8),
}

impl Acpi {
    pub fn can_power_off(&self) -> bool {
        self.s5.is_some() && self.pm1.0 != 0
    }
}

static ACPI: SpinLock<Option<Acpi>> = SpinLock::new(None);

/// The tables, if they were found.
pub fn get() -> Option<Acpi> {
    *ACPI.lock()
}

/// Where ISA interrupt `line` arrives: an override, or the same number
/// (edge triggered, active high) as on QEMU without tables.
pub fn isa_route(line: u32) -> IsaRoute {
    get().map_or(IsaRoute { gsi: line, active_low: false, level: false }, |a| a.isa[line as usize & 15])
}

/// A table at physical `addr`, `len` bytes, reachable by the kernel.
fn bytes(addr: u64, len: usize) -> Option<&'static [u8]> {
    let at = crate::arch::paging::map_device(addr, len as u64).ok()?;
    Some(unsafe { core::slice::from_raw_parts(at as *const u8, len) })
}

fn checksum_ok(b: &[u8]) -> bool {
    b.iter().fold(0u8, |s, &x| s.wrapping_add(x)) == 0
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn u64_at(b: &[u8], o: usize) -> u64 {
    u32_at(b, o) as u64 | (u32_at(b, o + 4) as u64) << 32
}

/// A whole table (header, then the rest) if its checksum holds.
fn table(addr: u64) -> Option<&'static [u8]> {
    let head = bytes(addr, 36)?;
    let len = u32_at(head, 4) as usize;
    if !(36..=1 << 20).contains(&len) {
        return None;
    }
    let all = bytes(addr, len)?;
    checksum_ok(all).then_some(all)
}

/// Looks for "RSD PTR " in the BIOS area, on 16-byte boundaries.
fn scan_bios() -> Option<u64> {
    (0xE_0000u64..0x10_0000).step_by(16).find(|&a| {
        let b = unsafe { core::slice::from_raw_parts(a as *const u8, 20) };
        &b[..8] == b"RSD PTR " && checksum_ok(b)
    })
}

/// Reads the tables; `rsdp` is what the boot loader gave (0 if nothing).
pub fn init(rsdp: u64) -> Result<Acpi, &'static str> {
    let rsdp = if rsdp != 0 { rsdp } else { scan_bios().ok_or("no RSDP")? };
    let r = bytes(rsdp, 36).ok_or("RSDP not reachable")?;
    if &r[..8] != b"RSD PTR " || !checksum_ok(&r[..20]) {
        return Err("bad RSDP");
    }
    let revision = r[15];
    // ACPI 2.0 and later: the XSDT has 64-bit pointers.
    let (root, wide) =
        if revision >= 2 && u64_at(r, 24) != 0 { (u64_at(r, 24), true) } else { (u32_at(r, 16) as u64, false) };
    let root = table(root).ok_or("bad RSDT/XSDT")?;
    let step = if wide { 8 } else { 4 };
    let mut a = Acpi {
        revision,
        cpus: 0,
        ioapic: 0xFEC0_0000,
        gsi_base: 0,
        overrides: 0,
        isa: core::array::from_fn(|i| IsaRoute { gsi: i as u32, active_low: false, level: false }),
        pm1: (0, 0),
        s5: None,
        smi: (0, 0),
    };
    for o in (36..root.len()).step_by(step) {
        if o + step > root.len() {
            break;
        }
        let ptr = if wide { u64_at(root, o) } else { u32_at(root, o) as u64 };
        let Some(t) = table(ptr) else { continue };
        match &t[..4] {
            b"APIC" => madt(t, &mut a),
            b"FACP" => fadt(t, &mut a),
            _ => {}
        }
    }
    a.cpus = a.cpus.max(1);
    *ACPI.lock() = Some(a);
    Ok(a)
}

fn madt(t: &[u8], a: &mut Acpi) {
    let mut first_ioapic = true;
    let mut o = 44;
    while o + 2 <= t.len() {
        let (kind, len) = (t[o], t[o + 1] as usize);
        if len < 2 || o + len > t.len() {
            break;
        }
        let e = &t[o..o + len];
        match kind {
            // Local APIC / x2APIC: enabled, or can be brought online.
            0 if len >= 8 && u32_at(e, 4) & 3 != 0 => a.cpus += 1,
            9 if len >= 16 && u32_at(e, 8) & 3 != 0 => a.cpus += 1,
            1 if len >= 12 && first_ioapic => {
                a.ioapic = u32_at(e, 4) as u64;
                a.gsi_base = u32_at(e, 8);
                first_ioapic = false;
            }
            // Interrupt source override (bus 0 = ISA).
            2 if len >= 10 && e[2] == 0 && e[3] < 16 => {
                let flags = u16::from_le_bytes([e[8], e[9]]);
                a.isa[e[3] as usize] =
                    IsaRoute { gsi: u32_at(e, 4), active_low: flags & 3 == 3, level: flags >> 2 & 3 == 3 };
                a.overrides += 1;
            }
            _ => {}
        }
        o += len;
    }
}

fn fadt(t: &[u8], a: &mut Acpi) {
    if t.len() < 76 {
        return;
    }
    a.smi = (u32_at(t, 48) as u16, t[52]);
    a.pm1 = (u32_at(t, 64) as u16, u32_at(t, 68) as u16);
    let dsdt = if t.len() >= 148 && u64_at(t, 140) != 0 { u64_at(t, 140) } else { u32_at(t, 40) as u64 };
    a.s5 = table(dsdt).and_then(s5);
}

/// Finds `Name(_S5_, Package() { a, b, ... })` in the DSDT's AML and
/// returns the two SLP_TYP values.
fn s5(dsdt: &[u8]) -> Option<(u8, u8)> {
    let at = dsdt.windows(4).position(|w| w == b"_S5_")?;
    // NameOp before the name (possibly with a root prefix).
    if at < 1 || !(dsdt[at - 1] == 0x08 || (at >= 2 && dsdt[at - 2] == 0x08 && dsdt[at - 1] == b'\\')) {
        return None;
    }
    let mut p = at + 4;
    if *dsdt.get(p)? != 0x12 {
        return None; // not a package
    }
    p += 1;
    p += (dsdt.get(p)? >> 6) as usize + 1; // package length
    p += 1; // element count
    let mut value = || -> Option<u8> {
        let v = match *dsdt.get(p)? {
            0x0A => {
                p += 1;
                *dsdt.get(p)?
            }
            0x00 => 0, // ZeroOp
            0x01 => 1, // OneOp
            v if v < 8 => v,
            _ => return None,
        };
        p += 1;
        Some(v)
    };
    let typ_a = value()?;
    let typ_b = value().unwrap_or(0);
    Some((typ_a, typ_b))
}

/// Switches the machine off (sleep state S5). Returns only if it did not.
pub fn power_off() {
    let Some(a) = get().filter(|a| a.can_power_off()) else { return };
    let (typ_a, typ_b) = a.s5.unwrap_or((0, 0));
    unsafe {
        // Still in legacy mode: ask the firmware for ACPI mode first.
        if inw(a.pm1.0) & 1 == 0 && a.smi.0 != 0 && a.smi.1 != 0 {
            outb(a.smi.0, a.smi.1);
            for _ in 0..300 {
                if inw(a.pm1.0) & 1 != 0 {
                    break;
                }
                crate::arch::delay_us(10_000);
            }
        }
        // Keep SCI_EN, set the sleep type and the enable bit.
        const SLP_EN: u16 = 1 << 13;
        let keep = inw(a.pm1.0) & 1;
        outw(a.pm1.0, keep | (typ_a as u16) << 10 | SLP_EN);
        if a.pm1.1 != 0 {
            outw(a.pm1.1, keep | (typ_b as u16) << 10 | SLP_EN);
        }
    }
    crate::arch::delay_us(100_000);
}
