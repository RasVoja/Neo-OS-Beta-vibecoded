//! The kernel's own page tables (4-level, x86_64).
//!
//! For now physical memory is identity mapped, from 0 up to at least 4 GiB
//! (so device registers below 4 GiB are reachable) or the end of RAM if that
//! is higher. 2 MiB pages are used everywhere except around the kernel image,
//! which gets 4 KiB pages so that single pages can be left out: the page
//! under the boot stack stays unmapped as a guard.

use crate::mem::{frame, PAGE_SIZE};
use core::arch::asm;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER: u64 = 1 << 2;
const HUGE: u64 = 1 << 7;
/// Bit 9 is free for the system: marks device memory, which is not RAM
/// and must never go back to the frame allocator.
const DEVICE: u64 = 1 << 9;
const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;
const SIZE_2M: u64 = 2 * 1024 * 1024;

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
    next_with(entry, 0)
}

/// Like `next`, also setting `extra` flags (USER) on the entry.
fn next_with(entry: &mut u64, extra: u64) -> &'static mut [u64; 512] {
    if *entry & PRESENT == 0 {
        *entry = new_table() | PRESENT | WRITABLE;
    }
    *entry |= extra;
    assert!(*entry & HUGE == 0, "huge page where a table was expected");
    table(*entry & ADDR_MASK)
}

fn idx(virt: u64, level: u32) -> usize {
    ((virt >> (12 + 9 * level)) & 0x1FF) as usize
}

pub struct AddressSpace {
    pml4: u64,
}

impl AddressSpace {
    pub fn new() -> Self {
        AddressSpace { pml4: new_table() }
    }

    pub fn map_4k(&mut self, virt: u64, phys: u64, flags: u64) {
        let pdpt = next(&mut table(self.pml4)[idx(virt, 3)]);
        let pd = next(&mut pdpt[idx(virt, 2)]);
        let pt = next(&mut pd[idx(virt, 1)]);
        pt[idx(virt, 0)] = phys | flags | PRESENT;
    }

    /// Maps a page that user mode (ring 3) may use.
    pub fn map_user_4k(&mut self, virt: u64, phys: u64, writable: bool) {
        assert!(is_user(virt), "{:#x} is not a user address", virt);
        let pdpt = next_with(&mut table(self.pml4)[idx(virt, 3)], USER);
        let pd = next_with(&mut pdpt[idx(virt, 2)], USER);
        let pt = next_with(&mut pd[idx(virt, 1)], USER);
        let w = if writable { WRITABLE } else { 0 };
        pt[idx(virt, 0)] = phys | w | USER | PRESENT;
    }

    /// Maps a page of device memory (a framebuffer, say) for the program:
    /// writable and never freed with the space. (Not marked no-execute:
    /// the kernel does not turn NX on yet.)
    pub fn map_user_device_4k(&mut self, virt: u64, phys: u64) {
        assert!(is_user(virt), "{:#x} is not a user address", virt);
        let pdpt = next_with(&mut table(self.pml4)[idx(virt, 3)], USER);
        let pd = next_with(&mut pdpt[idx(virt, 2)], USER);
        let pt = next_with(&mut pd[idx(virt, 1)], USER);
        pt[idx(virt, 0)] = phys | DEVICE | WRITABLE | USER | PRESENT;
    }

    /// Maps shared memory for user mode, writable; like device memory it is
    /// not freed with the space (its owner, `process::Shared`, frees it).
    pub fn map_user_shared_4k(&mut self, virt: u64, phys: u64) {
        self.map_user_device_4k(virt, phys);
    }

    /// True if `[virt, virt + len)` is mapped for user mode in this space
    /// (and writable by the program, when `write` is set).
    pub fn user_range_ok(&self, virt: u64, len: u64, write: bool) -> bool {
        let Some(end) = virt.checked_add(len) else { return false };
        if !is_user(virt) || (len > 0 && !is_user(end - 1)) {
            return false;
        }
        let mut p = virt & !(PAGE_SIZE - 1);
        while p < end {
            match self.user_entry(p) {
                Some(e) if !write || e & WRITABLE != 0 => {}
                _ => return false,
            }
            p += PAGE_SIZE;
        }
        true
    }

    fn user_entry(&self, virt: u64) -> Option<u64> {
        let mut t = table(self.pml4);
        for level in (1..=3).rev() {
            let e = t[idx(virt, level)];
            if e & PRESENT == 0 || e & USER == 0 || e & HUGE != 0 {
                return None;
            }
            t = table(e & ADDR_MASK);
        }
        let e = t[idx(virt, 0)];
        (e & PRESENT != 0 && e & USER != 0).then_some(e)
    }

    /// A new address space for a program: shares every kernel mapping
    /// (supervisor-only, so the program cannot touch them) and has an
    /// empty user part.
    pub fn new_user() -> Self {
        let space = AddressSpace::new();
        let kernel = table(kernel_space().pml4);
        let mine = table(space.pml4);
        for i in 0..512 {
            if !(USER_PML4_FIRST..=USER_PML4_LAST).contains(&i) {
                mine[i] = kernel[i];
            }
        }
        space
    }

    /// Frees every frame of the user part and the top-level table.
    /// The space must not be active.
    pub fn destroy_user(self) {
        fn free_level(t: u64, level: u32) {
            for &e in table(t).iter() {
                if e & PRESENT == 0 {
                    continue;
                }
                if level > 1 {
                    free_level(e & ADDR_MASK, level - 1);
                } else if e & DEVICE == 0 {
                    frame::free(e & ADDR_MASK);
                }
            }
            frame::free(t);
        }
        let top = table(self.pml4);
        for i in USER_PML4_FIRST..=USER_PML4_LAST {
            if top[i] & PRESENT != 0 {
                free_level(top[i] & ADDR_MASK, 3);
            }
        }
        frame::free(self.pml4);
    }

    pub fn root(&self) -> u64 {
        self.pml4
    }

    pub fn map_2m(&mut self, virt: u64, phys: u64, flags: u64) {
        let pdpt = next(&mut table(self.pml4)[idx(virt, 3)]);
        let pd = next(&mut pdpt[idx(virt, 2)]);
        pd[idx(virt, 1)] = phys | flags | PRESENT | HUGE;
    }

    /// Walks the tables; returns the physical address `virt` maps to.
    pub fn translate(&self, virt: u64) -> Option<u64> {
        let mut t = table(self.pml4);
        for level in (1..=3).rev() {
            let e = t[idx(virt, level)];
            if e & PRESENT == 0 {
                return None;
            }
            if e & HUGE != 0 {
                let size = 1u64 << (12 + 9 * level);
                return Some((e & ADDR_MASK & !(size - 1)) + (virt & (size - 1)));
            }
            t = table(e & ADDR_MASK);
        }
        let e = t[idx(virt, 0)];
        (e & PRESENT != 0).then(|| (e & ADDR_MASK) + (virt & (PAGE_SIZE - 1)))
    }

    pub fn activate(&self) {
        unsafe { asm!("mov cr3, {}", in(reg) self.pml4, options(nostack, preserves_flags)) };
    }
}

/// Programs live in PML4 slots 1..=255 (512 GiB up to the end of the lower
/// half). Slot 0 is the kernel's identity map; the upper half is kernel only.
const USER_PML4_FIRST: usize = 1;
const USER_PML4_LAST: usize = 255;
pub const USER_START: u64 = (USER_PML4_FIRST as u64) << 39;
pub const USER_END: u64 = ((USER_PML4_LAST as u64) + 1) << 39;

pub fn is_user(virt: u64) -> bool {
    (USER_START..USER_END).contains(&virt)
}

/// Makes `root` the active top-level table, if it is not already.
pub fn activate_root(root: u64) {
    let cur: u64;
    unsafe { asm!("mov {}, cr3", out(reg) cur, options(nomem, nostack, preserves_flags)) };
    if cur & ADDR_MASK != root {
        unsafe { asm!("mov cr3, {}", in(reg) root, options(nostack, preserves_flags)) };
    }
}

static mut KERNEL_SPACE: Option<AddressSpace> = None;

pub fn kernel_space() -> &'static AddressSpace {
    unsafe { (*core::ptr::addr_of!(KERNEL_SPACE)).as_ref().expect("paging not set up") }
}

/// Maps one 4 KiB page into the kernel's address space (writable).
/// The page must not be mapped yet, so no TLB flush is needed.
pub fn map_kernel_page(virt: u64, phys: u64) {
    let space = unsafe { (*core::ptr::addr_of_mut!(KERNEL_SPACE)).as_mut().expect("paging not set up") };
    assert!(space.translate(virt).is_none(), "page {:#x} already mapped", virt);
    space.map_4k(virt, phys, WRITABLE);
}

/// Builds and switches to the kernel's page tables. Returns the mapped size.
pub fn init(ram_end: u64, kernel_end: u64) -> u64 {
    let limit = ram_end.max(4 << 30).next_multiple_of(SIZE_2M);
    let small_end = kernel_end.next_multiple_of(SIZE_2M);
    let guard = guard_page();
    let mut space = AddressSpace::new();

    let mut a = 0;
    while a < limit {
        if a < small_end {
            for p in (a..a + SIZE_2M).step_by(PAGE_SIZE as usize) {
                if p != guard {
                    space.map_4k(p, p, WRITABLE);
                }
            }
        } else {
            space.map_2m(a, a, WRITABLE);
        }
        a += SIZE_2M;
    }
    space.activate();
    unsafe { *core::ptr::addr_of_mut!(KERNEL_SPACE) = Some(space) };
    limit
}
