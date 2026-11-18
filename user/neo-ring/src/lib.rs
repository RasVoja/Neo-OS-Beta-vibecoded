//! A one-way byte ring in shared memory, for messages too large for a
//! port (a port message carries 16 bytes). One side writes, the other
//! reads; neither waits on the other inside the ring. A port message, the
//! "doorbell", tells the reader there is something new, so it can sleep
//! in `recv` instead of spinning.
//!
//! The block starts with a 64-byte header: the bytes written so far
//! (`head`) and the bytes read so far (`tail`), both counting up forever
//! (64 bits never wrap in practice). The data follows. Only the writer
//! moves `head` and only the reader moves `tail`, so no lock is needed:
//! the writer publishes data with a Release store of `head`, the reader
//! sees it with an Acquire load, and the same the other way for `tail`.
//!
//! The kernel uses this crate too, to test the ring from the other side.

#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

/// Header size: head and tail on a line of their own.
const HEADER: usize = 64;

#[repr(C)]
struct Header {
    head: AtomicU64,
    tail: AtomicU64,
}

/// One end of a ring laid over a block of memory.
pub struct Ring {
    header: *const Header,
    data: *mut u8,
    size: u64,
}

impl Ring {
    /// Lays a ring over `bytes` bytes at `base`. The block must be zeroed
    /// when the ring is first made (a fresh shared block is), and both
    /// sides must pass the same size.
    ///
    /// # Safety
    /// `base` must point to `bytes` bytes of memory, 8-byte aligned, that
    /// stay mapped while the ring is used, and `bytes` must be more than 64.
    pub unsafe fn new(base: *mut u8, bytes: usize) -> Ring {
        Ring { header: base as *const Header, data: base.add(HEADER), size: (bytes - HEADER) as u64 }
    }

    fn header(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// How many bytes the ring can hold.
    pub fn capacity(&self) -> usize {
        self.size as usize
    }

    /// Bytes written and not read yet.
    pub fn used(&self) -> usize {
        let h = self.header();
        (h.head.load(Ordering::Acquire) - h.tail.load(Ordering::Acquire)) as usize
    }

    /// The writer's side: copies as much of `data` as fits and returns how
    /// much that was (0 when the ring is full).
    pub fn write(&self, data: &[u8]) -> usize {
        let h = self.header();
        let head = h.head.load(Ordering::Relaxed);
        let free = self.size - (head - h.tail.load(Ordering::Acquire));
        let n = (data.len() as u64).min(free) as usize;
        for (i, &b) in data[..n].iter().enumerate() {
            let at = ((head + i as u64) % self.size) as usize;
            unsafe { self.data.add(at).write_volatile(b) };
        }
        h.head.store(head + n as u64, Ordering::Release);
        n
    }

    /// The reader's side: copies up to `out.len()` bytes out and returns
    /// how many (0 when the ring is empty).
    pub fn read(&self, out: &mut [u8]) -> usize {
        let h = self.header();
        let tail = h.tail.load(Ordering::Relaxed);
        let have = h.head.load(Ordering::Acquire) - tail;
        let n = (out.len() as u64).min(have) as usize;
        for (i, b) in out[..n].iter_mut().enumerate() {
            let at = ((tail + i as u64) % self.size) as usize;
            *b = unsafe { self.data.add(at).read_volatile() };
        }
        h.tail.store(tail + n as u64, Ordering::Release);
        n
    }
}

/// The byte at position `i` of the test pattern both test sides use: it
/// changes every byte and does not repeat with the ring's size.
pub fn pattern(i: u64) -> u8 {
    (i ^ (i >> 8) ^ (i >> 16)).wrapping_mul(31) as u8
}
