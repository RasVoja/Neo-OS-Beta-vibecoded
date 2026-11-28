//! Physical frame allocator: hands out 4 KiB frames of RAM.
//!
//! A bitmap with one bit per frame (1 = in use), sized from the memory map
//! and stored in RAM itself. Everything starts as in use; `add_free`
//! releases the usable ranges from the firmware memory map and `reserve`
//! takes back what the kernel, the loader and the bitmap still occupy.
//!
//! Frames are only handed out below `limit`, the memory the current page
//! tables reach. At boot that is the first 1 GiB; it is raised once the
//! kernel's own page tables map all RAM.

use super::PAGE_SIZE;
use crate::sync::SpinLock;

/// What the boot page tables map; the bitmap must sit below this.
pub const BOOT_MAPPED: u64 = 1 << 30;

pub struct FrameAllocator {
    bits: *mut u64,
    words: usize,
    free: usize,
    /// Frames of usable RAM, free or not.
    usable: usize,
    /// Lowest word that may contain a free frame.
    hint: usize,
    /// Words below this may be handed out (memory that is mapped).
    limit_words: usize,
}

// The bitmap is owned by the allocator; access goes through the lock.
unsafe impl Send for FrameAllocator {}

impl FrameAllocator {
    pub const fn empty() -> Self {
        FrameAllocator { bits: core::ptr::null_mut(), words: 0, free: 0, usable: 0, hint: 0, limit_words: 0 }
    }

    /// Bytes of bitmap needed to cover RAM up to `ram_end`.
    pub fn bitmap_bytes(ram_end: u64) -> u64 {
        let frames = ram_end.div_ceil(PAGE_SIZE);
        frames.div_ceil(64) * 8
    }

    /// # Safety
    /// `at` must be identity mapped, writable RAM of `bitmap_bytes(ram_end)`
    /// bytes that nothing else uses. The caller must `reserve` it afterwards.
    pub unsafe fn init(&mut self, at: u64, ram_end: u64) {
        self.words = (Self::bitmap_bytes(ram_end) / 8) as usize;
        self.bits = at as *mut u64;
        self.bitmap().fill(u64::MAX);
        self.free = 0;
        self.hint = 0;
        self.set_limit(BOOT_MAPPED);
    }

    /// Allows frames below `mapped` to be handed out.
    pub fn set_limit(&mut self, mapped: u64) {
        self.limit_words = ((mapped / PAGE_SIZE / 64) as usize).min(self.words);
    }

    fn bitmap(&mut self) -> &mut [u64] {
        if self.bits.is_null() {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.bits, self.words) }
    }

    fn frames(&self) -> usize {
        self.words * 64
    }

    /// Frame numbers fully inside [start, end).
    fn frames_inside(&self, start: u64, end: u64) -> core::ops::Range<usize> {
        let first = start.div_ceil(PAGE_SIZE) as usize;
        let last = ((end / PAGE_SIZE) as usize).min(self.frames());
        first..last.max(first)
    }

    /// Frame numbers touching [start, end).
    fn frames_touching(&self, start: u64, end: u64) -> core::ops::Range<usize> {
        let first = (start / PAGE_SIZE) as usize;
        let last = (end.div_ceil(PAGE_SIZE) as usize).min(self.frames());
        first..last.max(first)
    }

    fn is_used(&mut self, f: usize) -> bool {
        self.bitmap()[f / 64] & (1 << (f % 64)) != 0
    }

    fn set_used(&mut self, f: usize, used: bool) {
        let was = self.is_used(f);
        if used && !was {
            self.bitmap()[f / 64] |= 1 << (f % 64);
            self.free -= 1;
        } else if !used && was {
            self.bitmap()[f / 64] &= !(1 << (f % 64));
            self.free += 1;
            self.hint = self.hint.min(f / 64);
        }
    }

    /// Marks the whole frames inside a usable RAM range as free.
    pub fn add_free(&mut self, start: u64, end: u64) {
        for f in self.frames_inside(start, end) {
            if self.is_used(f) {
                self.usable += 1;
            }
            self.set_used(f, false);
        }
    }

    /// Frames of usable RAM the firmware reported (in use or free).
    pub fn usable_frames(&self) -> usize {
        self.usable
    }

    /// Marks every frame touching [start, end) as in use.
    pub fn reserve(&mut self, start: u64, end: u64) {
        for f in self.frames_touching(start, end) {
            self.set_used(f, true);
        }
    }

    /// Returns the physical address of a free frame, lowest first.
    pub fn alloc(&mut self) -> Option<u64> {
        let (hint, limit) = (self.hint, self.limit_words);
        for w in hint..limit {
            let word = self.bitmap()[w];
            if word != u64::MAX {
                let f = w * 64 + (!word).trailing_zeros() as usize;
                self.set_used(f, true);
                self.hint = w;
                return Some(f as u64 * PAGE_SIZE);
            }
        }
        self.hint = limit;
        None
    }

    /// Returns the first of `n` free frames in a row (for devices that need
    /// physically contiguous memory), lowest first.
    pub fn alloc_contiguous(&mut self, n: usize) -> Option<u64> {
        let limit = self.limit_words * 64;
        let mut start = self.hint * 64;
        while start + n <= limit {
            match (start..start + n).find(|&f| self.is_used(f)) {
                Some(used) => start = used + 1,
                None => {
                    for f in start..start + n {
                        self.set_used(f, true);
                    }
                    return Some(start as u64 * PAGE_SIZE);
                }
            }
        }
        None
    }

    /// Gives a frame back. Freeing a frame twice is a kernel bug.
    pub fn free(&mut self, addr: u64) {
        let f = (addr / PAGE_SIZE) as usize;
        assert!(addr % PAGE_SIZE == 0 && f < self.frames(), "bad frame {:#x}", addr);
        assert!(self.is_used(f), "double free of frame {:#x}", addr);
        self.set_used(f, false);
    }

    pub fn free_frames(&self) -> usize {
        self.free
    }

    /// Physical memory the bitmap covers.
    pub fn managed_bytes(&self) -> u64 {
        self.frames() as u64 * PAGE_SIZE
    }
}

pub static FRAMES_ALLOC: SpinLock<FrameAllocator> = SpinLock::new(FrameAllocator::empty());

pub fn alloc() -> Option<u64> {
    FRAMES_ALLOC.lock().alloc()
}

pub fn alloc_contiguous(n: usize) -> Option<u64> {
    FRAMES_ALLOC.lock().alloc_contiguous(n)
}

pub fn free(addr: u64) {
    FRAMES_ALLOC.lock().free(addr)
}

pub fn free_frames() -> usize {
    FRAMES_ALLOC.lock().free_frames()
}

pub fn usable_frames() -> usize {
    FRAMES_ALLOC.lock().usable_frames()
}
