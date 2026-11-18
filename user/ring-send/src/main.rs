//! Sends one megabyte through a shared-memory ring, for the kernel's ring
//! self-test. It makes a 64 KiB shared block, lays a ring over it and
//! passes the block to the port it was given (handle 0) with `RING_OPEN`.
//! Then it writes the test pattern in pieces of odd sizes, so the data
//! wraps around the ring's end mid-piece, and rings the doorbell
//! (`RING_DATA`) after each piece. When the ring is full it waits a tick
//! for the reader. It ends with the number of times it found the ring full.

#![no_std]
#![no_main]

use neo_ring::{pattern, Ring};

const PORT: neo_rt::Handle = 0;
const PAGES: u64 = 16;
const TOTAL: u64 = 1 << 20;
const RING_OPEN: u64 = 1;
const RING_DATA: u64 = 2;

#[no_mangle]
fn neo_main() -> i64 {
    let Ok(shm) = neo_rt::share(PAGES) else { return -1 };
    let Ok(base) = neo_rt::map_device(shm) else { return -2 };
    let bytes = (PAGES * 4096) as usize;
    let ring = unsafe { Ring::new(base, bytes) };
    if neo_rt::send_handle(PORT, RING_OPEN, bytes as u64, shm).is_err() {
        return -3;
    }
    let (mut sent, mut full) = (0u64, 0i64);
    let mut piece = [0u8; 3001];
    while sent < TOTAL {
        let n = (piece.len() as u64).min(TOTAL - sent) as usize;
        for (i, b) in piece[..n].iter_mut().enumerate() {
            *b = pattern(sent + i as u64);
        }
        let mut done = 0;
        while done < n {
            let w = ring.write(&piece[done..n]);
            if w == 0 {
                full += 1;
                neo_rt::sleep(1);
            }
            done += w;
        }
        sent += n as u64;
        // A full port only means the reader has doorbells waiting already.
        let _ = neo_rt::try_send(PORT, RING_DATA, sent, 0);
    }
    full
}
