//! The PCI bus of a PC: finds every function, on every bus, through the
//! bridges the firmware has set up.
//!
//! In QEMU every device sits on bus 0, but a real PC puts most of them
//! behind PCI Express root ports and bridges: the USB controller of a
//! typical AMD board is on bus 3 or higher. So the drivers no longer look
//! at bus 0 only; they ask this module, which walks the tree once at boot
//! through the legacy configuration ports (0xCF8/0xCFC).
//!
//! It also makes a device's registers reachable: the kernel identity maps
//! memory up to 4 GiB or the end of RAM, but UEFI firmware often puts
//! 64-bit BARs far above that, so `map` maps those into a window in the
//! kernel half, uncached.

use crate::arch::{pci_read32, pci_write32};
use crate::sync::SpinLock;
use alloc::vec::Vec;

/// One PCI function.
#[derive(Clone, Copy, Debug)]
pub struct Function {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
    /// Device id in the high half, vendor id in the low half.
    pub id: u32,
    /// Class, subclass and programming interface (0x0C0330 for xHCI).
    pub class: u32,
}

/// A memory BAR: where it is and how big.
#[derive(Clone, Copy, Debug)]
pub struct Bar {
    pub addr: u64,
    pub size: u64,
}

const CLASS_BRIDGE: u32 = 0x0604;
const CAP_MSI: u8 = 0x05;
const CAP_MSIX: u8 = 0x11;

/// How a device's interrupts reach the CPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Msi,
    MsiX,
}

impl Function {
    pub fn read(&self, offset: u8) -> u32 {
        pci_read32(self.bus, self.dev, self.func, offset)
    }

    pub fn write(&self, offset: u8, val: u32) {
        pci_write32(self.bus, self.dev, self.func, offset, val)
    }

    /// Sets bits in the command register: 1 I/O, 2 memory, 4 bus master.
    pub fn enable(&self, bits: u32) {
        let cmd = self.read(0x04) & 0xFFFF;
        self.write(0x04, cmd | bits);
    }

    /// Where capability `id` starts in configuration space, if the
    /// function has it.
    pub fn capability(&self, id: u8) -> Option<u8> {
        if self.read(0x04) >> 16 & 0x10 == 0 {
            return None; // no capability list
        }
        let mut at = (self.read(0x34) & 0xFC) as u8;
        // A broken list could loop; there is room for 48 at most.
        for _ in 0..48 {
            if at < 0x40 {
                return None;
            }
            let v = self.read(at);
            if v as u8 == id {
                return Some(at);
            }
            at = (v >> 8) as u8 & 0xFC;
        }
        None
    }

    /// Sends the function's interrupts as one message: a write of `data`
    /// to `addr` (on a PC, a vector to a local APIC). MSI is used if the
    /// function has it, else entry 0 of its MSI-X table; `bar_base` maps a
    /// BAR for the table. The old interrupt pin is switched off. Returns
    /// which way, or `None` when the function can do neither.
    pub fn enable_msi(&self, addr: u64, data: u32, bar_base: impl Fn(u8) -> Option<usize>) -> Option<Signal> {
        if let Some(c) = self.capability(CAP_MSI) {
            let head = self.read(c);
            let control = head >> 16;
            let wide = control & 0x80 != 0;
            self.write(c + 4, addr as u32);
            if wide {
                self.write(c + 8, (addr >> 32) as u32);
                self.write(c + 12, data & 0xFFFF);
            } else {
                self.write(c + 8, data & 0xFFFF);
            }
            // One message (multiple message enable 0), enabled.
            self.write(c, ((control & !0x70) | 1) << 16 | (head & 0xFFFF));
            self.enable(1 << 10); // interrupt pin off
            return Some(Signal::Msi);
        }
        let c = self.capability(CAP_MSIX)?;
        let table = self.read(c + 4);
        let base = bar_base((table & 7) as u8)?;
        let entry = base + (table & !7) as usize;
        unsafe {
            let e = entry as *mut u32;
            e.write_volatile(addr as u32);
            e.add(1).write_volatile((addr >> 32) as u32);
            e.add(2).write_volatile(data);
            e.add(3).write_volatile(0); // not masked
        }
        let head = self.read(c);
        // Enabled (bit 15), the whole function not masked (bit 14).
        let control = ((head >> 16) & !(1 << 14)) | 1 << 15;
        self.write(c, control << 16 | (head & 0xFFFF));
        self.enable(1 << 10);
        Some(Signal::MsiX)
    }

    /// The I/O port base of BAR `n`, if it is an I/O BAR.
    pub fn io_bar(&self, n: u8) -> Option<u16> {
        let v = self.read(0x10 + 4 * n);
        (v & 1 == 1).then_some((v & 0xFFFC) as u16)
    }

    /// Memory BAR `n` (32 or 64 bits), with its size. Decoding is off for
    /// the moment it takes to size it.
    pub fn mem_bar(&self, n: u8) -> Option<Bar> {
        let off = 0x10 + 4 * n;
        let lo = self.read(off);
        if lo & 1 != 0 {
            return None;
        }
        let wide = lo & 0x6 == 0x4;
        let hi = if wide { self.read(off + 4) } else { 0 };
        let cmd = self.read(0x04) & 0xFFFF;
        self.write(0x04, cmd & !0x3);
        self.write(off, 0xFFFF_FFFF);
        let mut mask = (self.read(off) & !0xF) as u64;
        self.write(off, lo);
        if wide {
            self.write(off + 4, 0xFFFF_FFFF);
            mask |= (self.read(off + 4) as u64) << 32;
            self.write(off + 4, hi);
        } else {
            mask |= 0xFFFF_FFFF_0000_0000;
        }
        self.write(0x04, cmd);
        let addr = (hi as u64) << 32 | (lo & !0xF) as u64;
        let size = (!mask).wrapping_add(1);
        (addr != 0 && size != 0).then_some(Bar { addr, size })
    }
}

static FUNCTIONS: SpinLock<Vec<Function>> = SpinLock::new(Vec::new());

fn scan_bus(bus: u8, out: &mut Vec<Function>, seen: &mut [bool; 256]) {
    if seen[bus as usize] {
        return;
    }
    seen[bus as usize] = true;
    for dev in 0..32u8 {
        if pci_read32(bus, dev, 0, 0) & 0xFFFF == 0xFFFF {
            continue;
        }
        let multi = pci_read32(bus, dev, 0, 0x0C) >> 16 & 0x80 != 0;
        for func in 0..if multi { 8 } else { 1 } {
            let id = pci_read32(bus, dev, func, 0);
            if id & 0xFFFF == 0xFFFF {
                continue;
            }
            let class = pci_read32(bus, dev, func, 0x08) >> 8;
            out.push(Function { bus, dev, func, id, class });
            if class >> 8 == CLASS_BRIDGE {
                let secondary = (pci_read32(bus, dev, func, 0x18) >> 8) as u8;
                if secondary != 0 {
                    scan_bus(secondary, out, seen);
                }
            }
        }
    }
}

/// Walks the buses once; returns (functions, buses).
pub fn init() -> (usize, usize) {
    let mut out = Vec::new();
    let mut seen = [false; 256];
    scan_bus(0, &mut out, &mut seen);
    // A multi-function host bridge at 0:0 means one root bus per function.
    if pci_read32(0, 0, 0, 0x0C) >> 16 & 0x80 != 0 {
        for func in 1..8u8 {
            if pci_read32(0, 0, func, 0) & 0xFFFF != 0xFFFF {
                scan_bus(func, &mut out, &mut seen);
            }
        }
    }
    let buses = seen.iter().filter(|&&s| s).count();
    let n = out.len();
    *FUNCTIONS.lock() = out;
    (n, buses)
}

/// The first function `wanted` accepts.
pub fn find(wanted: impl Fn(&Function) -> bool) -> Option<Function> {
    FUNCTIONS.lock().iter().find(|f| wanted(f)).copied()
}

/// Makes a memory BAR reachable by the kernel; returns the address the
/// kernel reads its registers at.
pub fn map(bar: Bar) -> Result<usize, &'static str> {
    crate::arch::paging::map_device(bar.addr, bar.size).map(|a| a as usize)
}
