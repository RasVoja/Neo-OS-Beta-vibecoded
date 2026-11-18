//! Bytes typed on the serial console.
//!
//! The UART raises an interrupt when a byte arrives; the handler moves every
//! waiting byte into a small buffer and wakes the thread that waits for
//! input. A reader that finds the buffer empty parks until then, so nothing
//! polls the port any more. When the buffer is full, further bytes are
//! dropped (and counted).

use crate::sched;
use crate::sync::SpinLock;
use core::sync::atomic::{AtomicU64, Ordering};

const SIZE: usize = 256;

struct Ring {
    buf: [u8; SIZE],
    head: usize,
    len: usize,
}

static RING: SpinLock<Ring> = SpinLock::new(Ring { buf: [0; SIZE], head: 0, len: 0 });
/// Thread waiting for input, or `NOBODY`.
static WAITER: AtomicU64 = AtomicU64::new(NOBODY);
const NOBODY: u64 = u64::MAX;

/// Serial interrupts handled so far.
pub static IRQS: AtomicU64 = AtomicU64::new(0);
/// Bytes dropped because the buffer was full.
pub static DROPPED: AtomicU64 = AtomicU64::new(0);

/// Called from the serial interrupt (interrupts off).
pub fn irq() {
    IRQS.fetch_add(1, Ordering::Relaxed);
    {
        let mut r = RING.lock();
        while let Some(b) = crate::arch::getc() {
            r.put(b);
        }
    }
    wake();
}

/// Where keyboard bytes go instead of the console, while a window has the
/// focus: that window's event port (set by the display service).
static FOCUS: crate::sync::SpinLock<Option<alloc::sync::Arc<crate::ipc::Port>>> = crate::sync::SpinLock::new(None);

/// The tag of a typed byte sent to a window: `EVENT_KEY` in neo-rt.
pub const EVENT_KEY: u64 = 18;

/// Sends keyboard bytes to `port` from now on, or back to the console.
/// (The serial console always types into the console.)
pub fn set_focus(port: Option<alloc::sync::Arc<crate::ipc::Port>>) {
    *FOCUS.lock() = port;
}

/// The keyboard layout in use: 0 is US, 1 is Serbian Latin. It lives here,
/// not in a driver, so both keyboard drivers and the Bar (which shows it
/// and switches it on a click) agree on it.
static LAYOUT: AtomicU64 = AtomicU64::new(0);
/// Who hears about a new layout: the display service's port.
static LAYOUT_SINK: crate::sync::SpinLock<Option<alloc::sync::Arc<crate::ipc::Port>>> =
    crate::sync::SpinLock::new(None);
/// The tag of that message (`LAYOUT_CHANGED` in neo-rt); a0 is the layout.
pub const LAYOUT_CHANGED: u64 = 9;

pub fn layout() -> u64 {
    LAYOUT.load(Ordering::Relaxed)
}

/// Sets the layout (0 or 1) and tells the display service if it changed.
pub fn set_layout(layout: u64) {
    let layout = layout & 1;
    if LAYOUT.swap(layout, Ordering::Relaxed) != layout {
        if let Some(port) = LAYOUT_SINK.lock().as_ref() {
            port.try_send(crate::ipc::Message::new(LAYOUT_CHANGED, [layout, 0, 0]));
        }
    }
}

/// Sends layout changes to `port` from now on (and the layout now).
pub fn set_layout_sink(port: Option<alloc::sync::Arc<crate::ipc::Port>>) {
    if let Some(p) = &port {
        p.try_send(crate::ipc::Message::new(LAYOUT_CHANGED, [layout(), 0, 0]));
    }
    *LAYOUT_SINK.lock() = port;
}

/// Adds one byte, as typed on a keyboard (the keyboard service, or the
/// virtio keyboard): to the window in focus if there is one, else to the
/// console.
pub fn push(b: u8) {
    if let Some(port) = FOCUS.lock().as_ref() {
        // A slow window loses keys rather than stopping the keyboard.
        port.try_send(crate::ipc::Message::new(EVENT_KEY, [b as u64, 0, 0]));
        return;
    }
    push_console(b);
}

/// Adds one byte to the console input, whoever has the focus.
pub fn push_console(b: u8) {
    RING.lock().put(b);
    wake();
}

fn wake() {
    let w = WAITER.load(Ordering::Relaxed);
    if w != NOBODY {
        sched::unpark(w);
    }
}

impl Ring {
    fn put(&mut self, b: u8) {
        if self.len == SIZE {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let at = (self.head + self.len) % SIZE;
        self.buf[at] = b;
        self.len += 1;
    }
}

/// Moves up to `out.len()` buffered bytes into `out` without waiting;
/// returns how many.
pub fn take(out: &mut [u8]) -> usize {
    let mut r = RING.lock();
    let n = r.len.min(out.len());
    for o in out[..n].iter_mut() {
        *o = r.buf[r.head];
        r.head = (r.head + 1) % SIZE;
    }
    r.len -= n;
    n
}

/// Waits until at least one byte has been typed, then returns what is
/// buffered (at most `out.len()` bytes, and at least one).
pub fn read(out: &mut [u8]) -> usize {
    if out.is_empty() {
        return 0;
    }
    let me = sched::current_id();
    loop {
        // Register first: a byte that arrives between the check and the
        // park leaves an unpark behind, so the park returns at once.
        WAITER.store(me, Ordering::Relaxed);
        let n = take(out);
        if n > 0 {
            WAITER.store(NOBODY, Ordering::Relaxed);
            return n;
        }
        sched::park();
    }
}
