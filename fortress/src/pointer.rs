//! The pointer (mouse or tablet): where it is on the screen and which
//! buttons are down.
//!
//! Drivers report movement: relative (a PS/2 mouse, through the driver's
//! pointer capability) or absolute (a virtio tablet). The kernel keeps the
//! position inside the screen and tells the display service, which draws
//! the arrow, with a message on its control port: tag `MOVED`, first
//! argument x, second y plus the buttons times 65536 (bit 0 left, bit 1
//! right, bit 2 middle). If the
//! service is behind, a move is dropped rather than waited for; the next
//! one carries the new position anyway.

use crate::display::{HEIGHT, WIDTH};
use crate::ipc::{Message, Port};
use crate::sync::SpinLock;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

/// The message tag the display service gets.
pub const MOVED: u64 = 3;

struct State {
    x: i64,
    y: i64,
    buttons: u64,
    sink: Option<Arc<Port>>,
}

static STATE: SpinLock<State> =
    SpinLock::new(State { x: WIDTH as i64 / 2, y: HEIGHT as i64 / 2, buttons: 0, sink: None });

/// Pointer reports so far (for the boot log and tests).
pub static MOVES: AtomicU64 = AtomicU64::new(0);

/// Applies a report; one that changes nothing is not passed on.
fn update(f: impl FnOnce(&mut State)) {
    let mut s = STATE.lock();
    let before = (s.x, s.y, s.buttons);
    f(&mut s);
    s.x = s.x.clamp(0, WIDTH as i64 - 1);
    s.y = s.y.clamp(0, HEIGHT as i64 - 1);
    if (s.x, s.y, s.buttons) == before {
        return;
    }
    MOVES.fetch_add(1, Ordering::Relaxed);
    if let Some(port) = &s.sink {
        port.try_send(Message::new(MOVED, [s.x as u64, s.y as u64 | s.buttons << 16, 0]));
    }
}

/// A relative move (screen pixels; y grows downwards).
pub fn moved(dx: i64, dy: i64, buttons: u64) {
    update(|s| {
        s.x += dx;
        s.y += dy;
        s.buttons = buttons & 7;
    });
}

/// An absolute position, given in `0..range` on both axes.
#[cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]
pub fn absolute(x: Option<u64>, y: Option<u64>, buttons: u64, range: u64) {
    update(|s| {
        if let Some(x) = x {
            s.x = (x * WIDTH / range.max(1)) as i64;
        }
        if let Some(y) = y {
            s.y = (y * HEIGHT / range.max(1)) as i64;
        }
        s.buttons = buttons & 7;
    });
}

/// Where the pointer is and its buttons.
pub fn position() -> (i64, i64, u64) {
    let s = STATE.lock();
    (s.x, s.y, s.buttons)
}

/// Where moves go (the display service's port), or nowhere.
pub fn set_sink(port: Option<Arc<Port>>) {
    let mut s = STATE.lock();
    s.sink = port;
    if let Some(p) = &s.sink {
        p.try_send(Message::new(MOVED, [s.x as u64, s.y as u64 | s.buttons << 16, 0]));
    }
}
