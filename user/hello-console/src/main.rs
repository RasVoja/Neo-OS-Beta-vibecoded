//! Prints through the console service (handle 0, send right only) instead
//! of asking the kernel, and checks it cannot touch the hardware itself.

#![no_std]
#![no_main]

use core::fmt::Write;
use neo_rt::{ConsoleClient, Error};

#[no_mangle]
fn neo_main() -> i64 {
    let mut out = ConsoleClient(0);
    if writeln!(out, "Hello through the console service!").is_err() {
        return 101;
    }
    if writeln!(out, "6 x 7 = {} (a longer line, sent in pieces of 16 bytes)", 6 * 7).is_err() {
        return 102;
    }
    // A port handle is not an I/O handle, and there is no handle 1.
    if neo_rt::io_out(0, 0, b'!') != Err(Error::Denied) {
        return 103;
    }
    if neo_rt::io_in(1, 0) != Err(Error::BadHandle) {
        return 104;
    }
    0
}
