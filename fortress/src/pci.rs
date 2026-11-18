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
