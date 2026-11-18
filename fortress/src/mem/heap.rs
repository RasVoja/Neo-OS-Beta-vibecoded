//! Kernel heap: makes `alloc` (Box, Vec, String) work inside Fortress.
//!
//! A first-fit free list, kept sorted by address so that neighbouring free
//! blocks merge on release. Every block is a multiple of 16 bytes and
//! 16-byte aligned. The heap starts empty at `arch::HEAP_BASE` and grows by
//! mapping fresh frames, up to `MAX_HEAP`.

use super::{frame, PAGE_SIZE};
use crate::sync::SpinLock;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

const MAX_HEAP: u64 = 256 * 1024 * 1024;
const UNIT: usize = 16;

struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

pub struct Heap {
    head: *mut FreeBlock,
    /// End of the mapped part of the heap.
    top: u64,
    free: usize,
}

unsafe impl Send for Heap {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub mapped: usize,
    pub free: usize,
    pub blocks: usize,
}

impl Heap {
    const fn new() -> Self {
        Heap { head: null_mut(), top: crate::arch::HEAP_BASE, free: 0 }
    }

    fn size_of(layout: &Layout) -> (usize, usize) {
        let size = layout.size().max(UNIT).next_multiple_of(UNIT);
        let align = layout.align().max(UNIT);
        (size, align)
    }

    /// Puts [addr, addr + size) on the free list, merging with neighbours.
    unsafe fn release(&mut self, addr: usize, size: usize) {
        self.free += size;
        let mut prev: *mut FreeBlock = null_mut();
        let mut cur = self.head;
        while !cur.is_null() && (cur as usize) < addr {
            prev = cur;
            cur = (*cur).next;
        }
        debug_assert!(cur.is_null() || addr + size <= cur as usize, "heap double free");

        let block = addr as *mut FreeBlock;
        block.write(FreeBlock { size, next: cur });
        // merge with the following block
        if !cur.is_null() && addr + size == cur as usize {
            (*block).size += (*cur).size;
            (*block).next = (*cur).next;
        }
        // link in, merging with the previous block
        if prev.is_null() {
            self.head = block;
        } else if prev as usize + (*prev).size == addr {
            (*prev).size += (*block).size;
            (*prev).next = (*block).next;
        } else {
            (*prev).next = block;
        }
    }

    unsafe fn take(&mut self, size: usize, align: usize) -> *mut u8 {
        let mut prev: *mut FreeBlock = null_mut();
        let mut cur = self.head;
        while !cur.is_null() {
            let start = cur as usize;
            let block_end = start + (*cur).size;
            let at = start.next_multiple_of(align);
            let end = at + size;
            if end <= block_end {
                let next = (*cur).next;
                let front = at - start; // 0 or a multiple of 16
                let back = block_end - end;
                // unlink cur, then give back the unused ends
                if prev.is_null() {
                    self.head = next;
                } else {
                    (*prev).next = next;
                }
                self.free -= (*cur).size;
                if front > 0 {
                    self.release(start, front);
                }
                if back > 0 {
                    self.release(end, back);
                }
                return at as *mut u8;
            }
            prev = cur;
            cur = (*cur).next;
        }
        null_mut()
    }

    /// Maps enough new pages for a block of `size` with `align`.
    unsafe fn grow(&mut self, size: usize, align: usize) -> bool {
        let want = (size + align) as u64;
        let pages = want.div_ceil(PAGE_SIZE).max(16); // at least 64 KiB at a time
        let base = crate::arch::HEAP_BASE;
        if self.top + pages * PAGE_SIZE > base + MAX_HEAP {
            return false;
        }
        let start = self.top;
        for _ in 0..pages {
            let Some(f) = frame::alloc() else { return false };
            crate::arch::map_kernel_page(self.top, f);
            self.top += PAGE_SIZE;
        }
        self.release(start as usize, (self.top - start) as usize);
        true
    }

    pub fn stats(&self) -> Stats {
        let mut blocks = 0;
        let mut cur = self.head;
        while !cur.is_null() {
            blocks += 1;
            cur = unsafe { (*cur).next };
        }
        Stats { mapped: (self.top - crate::arch::HEAP_BASE) as usize, free: self.free, blocks }
    }
}

pub struct KernelHeap(SpinLock<Heap>);

unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = Heap::size_of(&layout);
        let mut h = self.0.lock();
        let mut p = h.take(size, align);
        if p.is_null() && h.grow(size, align) {
            p = h.take(size, align);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = Heap::size_of(&layout);
        self.0.lock().release(ptr as usize, size);
    }
}

#[global_allocator]
pub static HEAP: KernelHeap = KernelHeap(SpinLock::new(Heap::new()));

pub fn stats() -> Stats {
    HEAP.0.lock().stats()
}
