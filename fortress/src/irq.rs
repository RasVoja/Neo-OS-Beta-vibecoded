//! Device interrupts handed to drivers in user mode.
//!
//! A driver gets an interrupt line as a capability and waits on it with a
//! system call. The kernel's handler only counts the interrupt and wakes the
//! waiting thread; the driver then talks to its device through its own I/O
//! capability. The kernel can take a line back (`revoke`): a driver waiting
//! on it gets an error and can end cleanly.

use crate::sched;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Lines 0-15: the ISA interrupts of a PC; line 16, the USB controller's
/// message-signalled interrupt (MSI), which the kernel's own USB driver
/// waits on.
pub const LINES: usize = 17;
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub const USB: usize = 16;
const NOBODY: u64 = u64::MAX;

static PENDING: [AtomicU64; LINES] = [const { AtomicU64::new(0) }; LINES];
static WAITER: [AtomicU64; LINES] = [const { AtomicU64::new(NOBODY) }; LINES];
static REVOKED: [AtomicBool; LINES] = [const { AtomicBool::new(false) }; LINES];

/// Called from the interrupt handler for `line`.
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub fn fire(line: usize) {
    PENDING[line].fetch_add(1, Ordering::Relaxed);
    let w = WAITER[line].load(Ordering::Relaxed);
    if w != NOBODY {
        sched::unpark(w);
    }
}

/// How many times `line` fired since the last wait (without waiting).
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub fn pending(line: usize) -> u64 {
    PENDING[line].load(Ordering::Relaxed)
}

/// Makes `line` usable again before it is handed to a new driver.
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub fn claim(line: usize) {
    PENDING[line].store(0, Ordering::Relaxed);
    REVOKED[line].store(false, Ordering::Relaxed);
}

/// Takes the line back: the driver's current or next wait fails.
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub fn revoke(line: usize) {
    REVOKED[line].store(true, Ordering::Relaxed);
    let w = WAITER[line].load(Ordering::Relaxed);
    if w != NOBODY {
        sched::unpark(w);
    }
}

/// Blocks until one of `lines` has fired at least once since the last
/// wait; returns how many times in all. `None` if a line was taken back.
/// (A driver with two lines, like the PC's keyboard controller with its
/// keyboard and mouse, waits on both.)
pub fn wait(lines: &[usize]) -> Option<u64> {
    let me = sched::current_id();
    let leave = || lines.iter().for_each(|&l| WAITER[l].store(NOBODY, Ordering::Relaxed));
    loop {
        // Register first, so that an interrupt between the check and the
        // park leaves an unpark behind.
        let mut n = 0;
        for &line in lines {
            WAITER[line].store(me, Ordering::Relaxed);
            if REVOKED[line].load(Ordering::Relaxed) {
                leave();
                return None;
            }
            n += PENDING[line].swap(0, Ordering::Relaxed);
        }
        if n > 0 {
            leave();
            return Some(n);
        }
        sched::park();
    }
}
