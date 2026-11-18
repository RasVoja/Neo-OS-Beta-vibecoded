//! Kernel console: formatted text to whatever early output the arch provides.

use core::fmt::{self, Write};

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                crate::arch::putc(b'\r');
            }
            crate::arch::putc(b);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Single core, interrupts off: no lock needed yet.
    let _ = Console.write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => { $crate::console::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {{ $crate::console::_print(format_args!($($arg)*)); $crate::kprint!("\n"); }};
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
