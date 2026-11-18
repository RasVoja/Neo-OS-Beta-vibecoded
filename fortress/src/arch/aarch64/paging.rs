//! The kernel's own page tables (4 KiB granule, 4 levels, 48-bit, AArch64).
//!
//! Like on x86_64, physical memory is identity mapped from 0 up to at least
//! 4 GiB or the end of RAM, with 2 MiB blocks everywhere except around the
//! kernel image, which gets 4 KiB pages so the page under the boot stack can
//! stay unmapped as a guard. RAM is mapped as normal cached memory and
//! everything else as device memory.
//!
//! The kernel's top-level table serves both halves: TTBR0 (low addresses:
//! the identity map) and TTBR1 (high addresses: the kernel heap) point at
//! it, just as x86_64 uses one PML4. A program gets its own top-level table
//! in TTBR0 that shares the kernel's entries (reachable from EL1 only) and
//! adds its own part, top-level slots 1..=255, as on x86_64.

use crate::mem::{frame, PAGE_SIZE};
use core::arch::asm;

const VALID: u64 = 1 << 0;
/// At levels 0-2: points to a table. At level 3: a page.
const TABLE: u64 = 1 << 1;
const ATTR_DEVICE: u64 = 0 << 2;
const ATTR_NORMAL: u64 = 1 << 2;
const INNER_SHAREABLE: u64 = 3 << 8;
const ACCESSED: u64 = 1 << 10;
/// Access permissions: EL0 may use the page, and the page is read-only.
const AP_EL0: u64 = 1 << 6;
const AP_RO: u64 = 1 << 7;
const NOT_GLOBAL: u64 = 1 << 11;
const PXN: u64 = 1 << 53;
const UXN: u64 = 1 << 54;
/// Bit 55 is free for software: marks device memory, which is not RAM and
/// must never go back to the frame allocator.
const SW_DEVICE: u64 = 1 << 55;
const ADDR_MASK: u64 = 0x0000_FFFF_FFFF_F000;
const SIZE_2M: u64 = 2 * 1024 * 1024;

/// Kernel read-write RAM.
const RAM: u64 = ATTR_NORMAL | INNER_SHAREABLE | ACCESSED | UXN;
/// Kernel read-write device registers, never executable.
const DEVICE: u64 = ATTR_DEVICE | ACCESSED | PXN | UXN;

extern "C" {
    static boot_stack_guard: u8;
}

pub fn guard_page() -> u64 {
    core::ptr::addr_of!(boot_stack_guard) as u64
}

/// Physical memory is identity mapped, so a table's address is its pointer.
fn table(phys: u64) -> &'static mut [u64; 512] {
    unsafe { &mut *(phys as *mut [u64; 512]) }
}

fn new_table() -> u64 {
    let f = frame::alloc().expect("out of memory for page tables");
    table(f).fill(0);
    f
}

/// Returns the next-level table under `entry`, creating it if missing.
fn next(entry: &mut u64) -> &'static mut [u64; 512] {
    if *entry & VALID == 0 {
        *entry = new_table() | TABLE | VALID;
    }
    assert!(*entry & TABLE != 0, "block where a table was expected");
    table(*entry & ADDR_MASK)
}

fn idx(virt: u64, level: u32) -> usize {
    ((virt >> (12 + 9 * level)) & 0x1FF) as usize
}

pub struct AddressSpace {
    root: u64,
}

impl AddressSpace {
    pub fn new() -> Self {
        AddressSpace { root: new_table() }
    }

    pub fn map_4k(&mut self, virt: u64, phys: u64, attrs: u64) {
        let l1 = next(&mut table(self.root)[idx(virt, 3)]);
        let l2 = next(&mut l1[idx(virt, 2)]);
        let l3 = next(&mut l2[idx(virt, 1)]);
        l3[idx(virt, 0)] = phys | attrs | TABLE | VALID;
    }

    pub fn map_2m(&mut self, virt: u64, phys: u64, attrs: u64) {
        let l1 = next(&mut table(self.root)[idx(virt, 3)]);
        let l2 = next(&mut l1[idx(virt, 2)]);
        l2[idx(virt, 1)] = phys | attrs | VALID;
    }

    /// Walks the tables; returns the physical address `virt` maps to.
    pub fn translate(&self, virt: u64) -> Option<u64> {
        let mut t = table(self.root);
        for level in (1..=3).rev() {
            let e = t[idx(virt, level)];
            if e & VALID == 0 {
                return None;
            }
            if e & TABLE == 0 {
                let size = 1u64 << (12 + 9 * level);
                return Some((e & ADDR_MASK & !(size - 1)) + (virt & (size - 1)));
            }
            t = table(e & ADDR_MASK);
        }
        let e = t[idx(virt, 0)];
        (e & VALID != 0).then(|| (e & ADDR_MASK) + (virt & (PAGE_SIZE - 1)))
    }

    pub fn root(&self) -> u64 {
        self.root
    }

    /// Maps a page that the program (EL0) may use. Writable pages are
    /// never executable; read-only ones may hold code. The kernel can
    /// never execute either.
    pub fn map_user_4k(&mut self, virt: u64, phys: u64, writable: bool) {
        assert!(is_user(virt), "{:#x} is not a user address", virt);
        let access = if writable { AP_EL0 | UXN } else { AP_EL0 | AP_RO };
        let attrs = ATTR_NORMAL | INNER_SHAREABLE | ACCESSED | NOT_GLOBAL | PXN | access;
        self.map_4k(virt, phys, attrs);
    }

    /// Maps a page of device memory for the program: device attributes,
    /// writable, never executable, and never freed with the space.
    pub fn map_user_device_4k(&mut self, virt: u64, phys: u64) {
        assert!(is_user(virt), "{:#x} is not a user address", virt);
        let attrs = ATTR_DEVICE | ACCESSED | NOT_GLOBAL | PXN | UXN | AP_EL0 | SW_DEVICE;
        self.map_4k(virt, phys, attrs);
    }

    /// Maps shared memory for EL0: normal cached RAM, writable, never
    /// executable, and not freed with the space (its owner,
    /// `process::Shared`, frees it).
    pub fn map_user_shared_4k(&mut self, virt: u64, phys: u64) {
        assert!(is_user(virt), "{:#x} is not a user address", virt);
        let attrs = ATTR_NORMAL | INNER_SHAREABLE | ACCESSED | NOT_GLOBAL | PXN | UXN | AP_EL0 | SW_DEVICE;
        self.map_4k(virt, phys, attrs);
    }

    /// True if `[virt, virt + len)` is mapped for EL0 in this space (and
    /// writable by the program, when `write` is set).
    pub fn user_range_ok(&self, virt: u64, len: u64, write: bool) -> bool {
        let Some(end) = virt.checked_add(len) else { return false };
        if !is_user(virt) || (len > 0 && !is_user(end - 1)) {
            return false;
        }
        let mut p = virt & !(PAGE_SIZE - 1);
        while p < end {
            match self.user_entry(p) {
                Some(e) if !write || e & AP_RO == 0 => {}
                _ => return false,
            }
            p += PAGE_SIZE;
        }
        true
    }

    fn user_entry(&self, virt: u64) -> Option<u64> {
        let mut t = table(self.root);
        for level in (1..=3).rev() {
            let e = t[idx(virt, level)];
            if e & VALID == 0 || e & TABLE == 0 {
                return None;
            }
            t = table(e & ADDR_MASK);
        }
        let e = t[idx(virt, 0)];
        (e & VALID != 0 && e & AP_EL0 != 0).then_some(e)
    }

    /// A new address space for a program: shares every kernel entry and
    /// has an empty user part.
    pub fn new_user() -> Self {
        let space = AddressSpace::new();
        let kernel = table(kernel_space().root);
        let mine = table(space.root);
        for i in 0..512 {
            if !(USER_L0_FIRST..=USER_L0_LAST).contains(&i) {
                mine[i] = kernel[i];
            }
        }
        publish();
        space
    }

    /// Frees every frame of the user part and the top-level table.
    /// The space must not be active.
    pub fn destroy_user(self) {
        fn free_level(t: u64, level: u32) {
            for &e in table(t).iter() {
                if e & VALID == 0 {
                    continue;
                }
                if level > 1 {
                    free_level(e & ADDR_MASK, level - 1);
                } else if e & SW_DEVICE == 0 {
                    frame::free(e & ADDR_MASK);
                }
            }
            frame::free(t);
        }
        let top = table(self.root);
        for i in USER_L0_FIRST..=USER_L0_LAST {
            if top[i] & VALID != 0 {
                free_level(top[i] & ADDR_MASK, 3);
            }
        }
        frame::free(self.root);
    }
}

/// Programs live in top-level slots 1..=255 (512 GiB up to 128 TiB), the
/// same addresses as on x86_64. Slot 0 is the kernel's identity map.
const USER_L0_FIRST: usize = 1;
const USER_L0_LAST: usize = 255;
pub const USER_START: u64 = (USER_L0_FIRST as u64) << 39;
pub const USER_END: u64 = ((USER_L0_LAST as u64) + 1) << 39;

pub fn is_user(virt: u64) -> bool {
    (USER_START..USER_END).contains(&virt)
}

/// Makes `root` the program part (TTBR0) of the address space, if it is
/// not already, and drops stale translations.
pub fn activate_root(root: u64) {
    let cur: u64;
    unsafe { asm!("mrs {}, ttbr0_el1", out(reg) cur, options(nomem, nostack)) };
    if cur & ADDR_MASK != root {
        unsafe {
            asm!(
                "msr ttbr0_el1, {}",
                "isb",
                "tlbi vmalle1",
                "dsb ish",
                "isb",
                in(reg) root,
                options(nostack)
            )
        };
    }
}

static mut KERNEL_SPACE: Option<AddressSpace> = None;

pub fn kernel_space() -> &'static AddressSpace {
    unsafe { (*core::ptr::addr_of!(KERNEL_SPACE)).as_ref().expect("paging not set up") }
}

/// Makes new table entries visible to the table walker.
fn publish() {
    unsafe { asm!("dsb ishst", "isb", options(nostack)) };
}

/// Maps one 4 KiB page of RAM into the kernel's address space (writable).
/// The page must not be mapped yet, so no TLB flush is needed.
pub fn map_kernel_page(virt: u64, phys: u64) {
    let space = unsafe { (*core::ptr::addr_of_mut!(KERNEL_SPACE)).as_mut().expect("paging not set up") };
    assert!(space.translate(virt).is_none(), "page {:#x} already mapped", virt);
    space.map_4k(virt, phys, RAM | PXN);
    publish();
}

/// Builds and switches to the kernel's page tables. `ram` lists the RAM
/// regions (normal memory); everything else up to the limit is device
/// memory. Returns the mapped size.
pub fn init(ram: &[(u64, u64)], kernel_start: u64, kernel_end: u64) -> u64 {
    let ram_end = ram.iter().map(|r| r.1).max().unwrap_or(0);
    let limit = ram_end.max(4 << 30).next_multiple_of(SIZE_2M);
    let small_start = kernel_start & !(SIZE_2M - 1);
    let small_end = kernel_end.next_multiple_of(SIZE_2M);
    let is_ram = |a: u64| ram.iter().any(|&(s, e)| s <= a && a + SIZE_2M <= e);
    let guard = guard_page();
    let mut space = AddressSpace::new();

    let mut a = 0;
    while a < limit {
        if (small_start..small_end).contains(&a) {
            for p in (a..a + SIZE_2M).step_by(PAGE_SIZE as usize) {
                if p != guard {
                    space.map_4k(p, p, RAM);
                }
            }
        } else {
            space.map_2m(a, a, if is_ram(a) { RAM } else { DEVICE });
        }
        a += SIZE_2M;
    }
    publish();

    // 48-bit addresses in both halves (T0SZ = T1SZ = 16), 4 KiB granules,
    // inner shareable write-back table walks, physical range as the CPU
    // reports it.
    let mmfr0: u64;
    unsafe { asm!("mrs {}, id_aa64mmfr0_el1", out(reg) mmfr0, options(nomem, nostack)) };
    let walk = (1 << 8) | (1 << 10) | (3 << 12);
    let tcr = 16 | walk | (16 << 16) | (walk << 16) | (0b10 << 30) | ((mmfr0 & 7) << 32);
    unsafe {
        asm!(
            "msr ttbr0_el1, {root}",
            "msr ttbr1_el1, {root}",
            "msr tcr_el1, {tcr}",
            "isb",
            "tlbi vmalle1",
            "dsb ish",
            "isb",
            root = in(reg) space.root(),
            tcr = in(reg) tcr,
            options(nostack)
        );
        *core::ptr::addr_of_mut!(KERNEL_SPACE) = Some(space);
    }
    limit
}
