//! The console service. It is the only program that touches the serial port:
//! handle 0 receives requests, handle 1 is the I/O range of the port (COM1).
//! Other programs send text to it (see `neo_rt::console_write`); the kernel
//! no longer has to print for them.

#![no_std]
#![no_main]

use neo_rt::{Error, Handle, CONSOLE_QUIT, CONSOLE_TEXT};

const REQUESTS: Handle = 0;
const SERIAL: Handle = 1;
/// UART registers: transmit holding register and line status register.
const THR: u16 = 0;
const LSR: u16 = 5;
const LSR_THR_EMPTY: u8 = 0x20;

fn put(b: u8) -> Result<(), Error> {
    while neo_rt::io_in(SERIAL, LSR)? & LSR_THR_EMPTY == 0 {}
    neo_rt::io_out(SERIAL, THR, b)
}

#[no_mangle]
fn neo_main() -> i64 {
    // The port range is ours, but only those 8 bytes of it.
    if neo_rt::io_in(SERIAL, 8) != Err(Error::Denied) {
        return 101;
    }
    let mut chars: u64 = 0;
    loop {
        let m = match neo_rt::recv(REQUESTS) {
            Ok(m) => m,
            Err(_) => return 102,
        };
        match m.tag & 0xFF {
            CONSOLE_TEXT => {
                let len = ((m.tag >> 8) & 0xFF).min(16) as usize;
                let mut buf = [0u8; 16];
                buf[..8].copy_from_slice(&m.a0.to_le_bytes());
                buf[8..].copy_from_slice(&m.a1.to_le_bytes());
                for &b in &buf[..len] {
                    if b == b'\n' && put(b'\r').is_err() {
                        return 103;
                    }
                    if put(b).is_err() {
                        return 103;
                    }
                }
                chars += len as u64;
            }
            CONSOLE_QUIT => return chars as i64,
            _ => {}
        }
    }
}
