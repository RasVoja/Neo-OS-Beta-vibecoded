//! Kernel console: formatted text to whatever early output the arch provides.
//!
//! Everything the kernel prints is also kept in a log in memory (the last
//! 64 KiB), which Neo DOS shows with BOOTLOG: on a real PC there is no
//! serial line to read the boot messages from, only the screen.

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Console;

const LOG_SIZE: usize = 64 * 1024;
static mut LOG: [u8; LOG_SIZE] = [0; LOG_SIZE];
/// Bytes ever written to the log; the last `LOG_SIZE` are kept.
static LOG_TOTAL: AtomicUsize = AtomicUsize::new(0);

fn log(b: u8) {
    let n = LOG_TOTAL.fetch_add(1, Ordering::Relaxed);
    // Single core: one writer at a time, as for the output itself.
    unsafe { (*core::ptr::addr_of_mut!(LOG))[n % LOG_SIZE] = b };
}

/// Copies the log, from `offset` bytes after its oldest kept byte, into
/// `out`; returns how many bytes were copied (0 at the end).
pub fn read_log(offset: usize, out: &mut [u8]) -> usize {
    let total = LOG_TOTAL.load(Ordering::Relaxed);
    let first = total.saturating_sub(LOG_SIZE);
    let n = (total - first).saturating_sub(offset).min(out.len());
    for (i, o) in out[..n].iter_mut().enumerate() {
        *o = unsafe { (*core::ptr::addr_of!(LOG))[(first + offset + i) % LOG_SIZE] };
    }
    n
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                crate::arch::putc(b'\r');
            }
            crate::arch::putc(b);
            log(b);
            crate::fbcon::put(b);
        }
        Ok(())
    }
}

/// Set while a thread prints a line, so two threads (the USB thread and the
/// start-up code) do not mix their lines. A thread that cannot get it after
/// a while (the holder was stopped by a fault, or this is a fault handler)
/// prints anyway: a mixed line beats a silent screen.
static PRINTING: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut spins = 0u32;
    while PRINTING.swap(true, Ordering::Acquire) {
        spins += 1;
        if spins > 200_000 {
            let _ = Console.write_fmt(args);
            return;
        }
        core::hint::spin_loop();
    }
    let _ = Console.write_fmt(args);
    PRINTING.store(false, Ordering::Release);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => { $crate::console::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {{ $crate::console::_print(format_args!("{}\n", format_args!($($arg)*))); }};
}

/// Where programs' console output is copied, if a terminal window shows it.
static TEE: crate::sync::SpinLock<Option<alloc::sync::Arc<crate::ipc::Port>>> = crate::sync::SpinLock::new(None);

/// The tag of a piece of console text sent to the terminal: `TEXT + n`
/// for n bytes (1-16) packed little-endian in the two arguments.
pub const TEXT: u64 = 0x40;

/// Copies what programs print to `port` from now on, or stops with None.
pub fn set_tee(port: Option<alloc::sync::Arc<crate::ipc::Port>>) {
    *TEE.lock() = port;
}

/// Passes printed bytes on to the terminal, 16 at a time. A terminal that
/// falls behind loses text rather than stopping the program.
pub fn tee(bytes: &[u8]) {
    let Some(port) = TEE.lock().clone() else { return };
    for chunk in bytes.chunks(16) {
        let mut w = [0u8; 16];
        w[..chunk.len()].copy_from_slice(chunk);
        let (a, b) = (
            u64::from_le_bytes(w[..8].try_into().unwrap_or([0; 8])),
            u64::from_le_bytes(w[8..].try_into().unwrap_or([0; 8])),
        );
        port.try_send(crate::ipc::Message::new(TEXT + chunk.len() as u64, [a, b, 0]));
    }
}
