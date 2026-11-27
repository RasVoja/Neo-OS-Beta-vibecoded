//! Self-test for shared-memory rings (user/neo-ring): a program sends one
//! megabyte to the kernel through a 64 KiB ring, ringing a port as a
//! doorbell. The kernel reads the ring from the other side, through its
//! identity map of RAM, and checks every byte. Shared by both
//! architectures.

use crate::ipc::{Message, Port};
use crate::process::{self, Cap, RIGHT_SEND};
use crate::{mem, programs, sched};
use alloc::string::String;
use neo_ring::{pattern, Ring};

const TOTAL: u64 = 1 << 20;
const RING_OPEN: u64 = 1;
const RING_DATA: u64 = 2;

/// Waits for a message, giving up after about `ticks` timer ticks (so a
/// program that died does not hang the self-test).
fn recv_within(port: &Port, ticks: u64) -> Option<Message> {
    for _ in 0..ticks {
        if let Some(m) = port.try_recv() {
            return Some(m);
        }
        sched::sleep(1);
    }
    None
}

pub fn ring_test() -> (bool, String) {
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let port = Port::new(64);
    let caps = alloc::vec![Cap::new(port.clone(), RIGHT_SEND)];
    let id = match process::spawn_elf("ring-send", programs::ring_send_elf(), caps, false) {
        Ok(id) => id,
        Err(e) => return (false, alloc::format!("refused: {}", e)),
    };

    let (mut got, mut bad, mut doorbells) = (0u64, 0u64, 0u64);
    let mut size = 0;
    let open = recv_within(&port, 200);
    if let Some(Message { tag: RING_OPEN, args: [bytes, ..], cap: Some(Cap::Shared(block)), .. }) = open {
        size = bytes;
        if bytes > 64 && bytes <= block.pages * mem::PAGE_SIZE {
            // RAM is identity mapped for the kernel.
            let ring = unsafe { Ring::new(block.phys as *mut u8, bytes as usize) };
            let mut buf = [0u8; 4096];
            while got < TOTAL {
                let n = ring.read(&mut buf);
                if n == 0 {
                    // Empty: sleep until the writer rings.
                    match recv_within(&port, 200) {
                        Some(Message { tag: RING_DATA, .. }) => doorbells += 1,
                        _ => break,
                    }
                    continue;
                }
                for (i, &b) in buf[..n].iter().enumerate() {
                    if b != pattern(got + i as u64) {
                        bad += 1;
                    }
                }
                got += n as u64;
            }
        }
        // `block` is dropped here; the frames go when the program's handle
        // goes too.
    }
    let full = process::wait(id);
    // Doorbells still queued.
    while port.try_recv().is_some() {}
    drop(port);

    let mut ok = got == TOTAL && bad == 0 && full >= 0;
    ok &= crate::thread_tests::wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{} KiB through a {} KiB ring, {} bytes wrong, {} doorbells, writer waited {} times, {} frames leaked",
            got / 1024,
            size / 1024,
            bad,
            doorbells,
            full,
            leaked
        ),
    )
}
