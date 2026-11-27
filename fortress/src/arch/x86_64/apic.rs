//! Interrupt controllers and the timer.
//!
//! The legacy 8259 PICs are moved out of the way (vectors 0xF0-0xFF) and
//! masked. The local APIC timer is calibrated against the PIT and then runs
//! periodically at `HZ` on vector `TIMER_VECTOR`.

use super::{inb, outb};
use core::arch::asm;

pub const TIMER_VECTOR: usize = 32;
pub const SPURIOUS_VECTOR: usize = 0xFF;
/// ISA interrupts, routed through the I/O APIC to vector 32 + line.
pub const ISA_BASE_VECTOR: usize = 32;
/// The serial port (ISA IRQ 4).
pub const SERIAL_VECTOR: usize = ISA_BASE_VECTOR + 4;
/// The PS/2 keyboard (ISA IRQ 1).
pub const KEYBOARD_VECTOR: usize = ISA_BASE_VECTOR + 1;
/// The PS/2 mouse (ISA line 12).
pub const MOUSE_VECTOR: usize = ISA_BASE_VECTOR + 12;
/// The USB controller, by MSI or MSI-X (above the ISA range).
pub const USB_VECTOR: usize = 48;
pub const HZ: u32 = 100;

const REG_TPR: usize = 0x80;
const REG_EOI: usize = 0xB0;
const REG_SVR: usize = 0xF0;
const REG_LVT_TIMER: usize = 0x320;
const REG_TIMER_INIT: usize = 0x380;
const REG_TIMER_CUR: usize = 0x390;
const REG_TIMER_DIV: usize = 0x3E0;

static mut BASE: u64 = 0xFEE0_0000;

fn rdmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    unsafe { asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nomem, nostack)) };
    (hi as u64) << 32 | lo as u64
}

fn read(reg: usize) -> u32 {
    unsafe { ((BASE as usize + reg) as *const u32).read_volatile() }
}

fn write(reg: usize, v: u32) {
    unsafe { ((BASE as usize + reg) as *mut u32).write_volatile(v) }
}

/// Remaps both PICs to 0xF0-0xFF (so a stray PIC interrupt can never look
/// like a CPU exception) and masks every line.
fn disable_pic() {
    unsafe {
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, 0xF0);
        outb(0xA1, 0xF8);
        outb(0x21, 0x04);
        outb(0xA1, 0x02);
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        outb(0x21, 0xFF);
        outb(0xA1, 0xFF);
    }
}

/// Counts LAPIC timer ticks (divider 16) during 10 ms measured by PIT channel 2.
fn calibrate() -> u32 {
    const PIT_HZ: u32 = 1_193_182;
    let count = (PIT_HZ / 100) as u16;
    unsafe {
        let gate = inb(0x61);
        outb(0x61, (gate & !0x02) | 0x01); // gate on, speaker off
        outb(0x43, 0xB0); // channel 2, lobyte/hibyte, mode 0
        outb(0x42, count as u8);
        outb(0x42, (count >> 8) as u8);
        let g = inb(0x61); // restart the count
        outb(0x61, g & !0x01);
        outb(0x61, g | 0x01);

        write(REG_TIMER_DIV, 0x3);
        write(REG_TIMER_INIT, u32::MAX);
        while inb(0x61) & 0x20 == 0 {}
        let elapsed = u32::MAX - read(REG_TIMER_CUR);
        write(REG_TIMER_INIT, 0);
        outb(0x61, gate);
        elapsed
    }
}

/// Returns the LAPIC timer frequency (after the divider) in Hz.
pub fn init() -> u64 {
    disable_pic();
    unsafe { BASE = rdmsr(0x1B) & 0xFFFF_F000 };
    write(REG_TPR, 0);
    write(REG_SVR, 0x100 | SPURIOUS_VECTOR as u32);

    let per_10ms = calibrate();
    let per_tick = per_10ms * 100 / HZ;
    write(REG_TIMER_DIV, 0x3);
    write(REG_LVT_TIMER, TIMER_VECTOR as u32 | (1 << 17)); // periodic
    write(REG_TIMER_INIT, per_tick.max(1));
    per_10ms as u64 * 100
}

/// The message (address and data) a PCI device writes to raise `vector`
/// on this CPU: fixed delivery, edge triggered.
pub fn msi_message(vector: usize) -> (u64, u32) {
    let id = read(0x20) >> 24;
    (0xFEE0_0000 | (id as u64) << 12, vector as u32)
}

pub fn eoi() {
    write(REG_EOI, 0);
}

/// The I/O APIC: where the ACPI tables say, or where QEMU and most PCs
/// put it.
fn ioapic() -> (usize, u32) {
    let (addr, base) = crate::acpi::get().map_or((0xFEC0_0000, 0), |a| (a.ioapic, a.gsi_base));
    let at = crate::arch::paging::map_device(addr, 0x20).unwrap_or(addr);
    (at as usize, base)
}

fn ioapic_write(at: usize, reg: u32, v: u32) {
    unsafe {
        (at as *mut u32).write_volatile(reg);
        ((at + 0x10) as *mut u32).write_volatile(v);
    }
}

/// Sends ISA interrupt `line` to this CPU on vector 32 + `line`. ISA
/// interrupts are edge triggered and active high, unless the ACPI tables
/// override that (and the GSI the line arrives on). (Line 0, the PIT, is
/// never routed: vector 32 is the local timer.)
pub fn route_isa(line: u32) {
    assert!(line > 0 && line < 16);
    let route = crate::acpi::isa_route(line);
    let (at, base) = ioapic();
    let Some(pin) = route.gsi.checked_sub(base) else { return };
    let id = read(0x20) >> 24;
    let entry = 0x10 + 2 * pin;
    let mut low = ISA_BASE_VECTOR as u32 + line;
    if route.active_low {
        low |= 1 << 13;
    }
    if route.level {
        low |= 1 << 15;
    }
    ioapic_write(at, entry + 1, id << 24);
    ioapic_write(at, entry, low);
}
