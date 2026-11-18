//! The keyboard service: the PS/2 controller's driver (keyboard and
//! mouse), running in user mode.
//!
//! Handle 0 is the I/O range of the PS/2 controller (ports 0x60-0x64),
//! handle 1 is its keyboard interrupt line (IRQ 1) and handle 2 lets it add
//! bytes to what the console reads. The kernel only counts the interrupt;
//! reading the scan codes and turning them into characters happens here.
//!
//! If it also gets handle 3, the mouse's interrupt line (IRQ 12), and
//! handle 4, a pointer capability, it runs the mouse too: it turns the
//! mouse's 3-byte packets into pointer moves. Bytes from the mouse are told
//! apart from keyboard bytes by bit 5 of the status register.
//!
//! The controller translates to scan code set 1, which is what this reads.
//! The neo-keys library turns key numbers into characters: US English or
//! Serbian Latin, switched with Left Alt + Shift.
//! The service ends when the kernel takes the interrupt line back, and
//! returns how many bytes it typed (č and the other Serbian letters are
//! two bytes each in UTF-8).

#![no_std]
#![no_main]

use neo_rt::{Error, Handle};

const PORTS: Handle = 0;
const IRQ: Handle = 1;
const TYPING: Handle = 2;
const MOUSE_IRQ: Handle = 3;
const POINTER: Handle = 4;
/// Offsets inside the port range: data at 0x60, status/command at 0x64.
const DATA: u16 = 0;
const STATUS: u16 = 4;
const OUTPUT_FULL: u8 = 1;
const INPUT_FULL: u8 = 2;
const FROM_MOUSE: u8 = 0x20;

fn wait_input_empty() -> Result<(), Error> {
    for _ in 0..100_000 {
        if neo_rt::io_in(PORTS, STATUS)? & INPUT_FULL == 0 {
            return Ok(());
        }
    }
    Err(Error::Denied)
}

fn command(cmd: u8) -> Result<(), Error> {
    wait_input_empty()?;
    neo_rt::io_out(PORTS, STATUS, cmd)
}

/// Turns on the keyboard's interrupt and scan code translation.
fn init() -> Result<(), Error> {
    // Whatever arrived before we started.
    while neo_rt::io_in(PORTS, STATUS)? & OUTPUT_FULL != 0 {
        neo_rt::io_in(PORTS, DATA)?;
    }
    command(0x20)?; // read the configuration byte
    let mut tries = 0;
    while neo_rt::io_in(PORTS, STATUS)? & OUTPUT_FULL == 0 {
        tries += 1;
        if tries > 100_000 {
            return Err(Error::Denied);
        }
    }
    let config = neo_rt::io_in(PORTS, DATA)?;
    // Bit 0: keyboard interrupt on; bit 4: clock off (clear it); bit 6: translate.
    let config = (config | 0x01 | 0x40) & !0x10;
    command(0x60)?;
    wait_input_empty()?;
    neo_rt::io_out(PORTS, DATA, config)?;
    command(0xAE) // enable the keyboard port
}

/// Waits for a byte from the controller (an answer to a command).
fn read_answer() -> Result<u8, Error> {
    for _ in 0..100_000 {
        if neo_rt::io_in(PORTS, STATUS)? & OUTPUT_FULL != 0 {
            return neo_rt::io_in(PORTS, DATA);
        }
    }
    Err(Error::Denied)
}

/// Sends a byte to the mouse (through command 0xD4) and takes its 0xFA.
fn to_mouse(b: u8) -> Result<(), Error> {
    command(0xD4)?;
    wait_input_empty()?;
    neo_rt::io_out(PORTS, DATA, b)?;
    match read_answer()? {
        0xFA => Ok(()),
        _ => Err(Error::Denied),
    }
}

/// Turns on the mouse port, its interrupt and the mouse's reports. Fails
/// at once without the mouse handles.
fn init_mouse() -> Result<(), Error> {
    neo_rt::pointer_move(POINTER, 0, 0, 0)?;
    command(0xA8)?; // enable the mouse port
    command(0x20)?;
    let config = read_answer()?;
    command(0x60)?;
    wait_input_empty()?;
    neo_rt::io_out(PORTS, DATA, (config | 0x02) & !0x20)?; // IRQ 12 on, clock on
    to_mouse(0xF6)?; // defaults
    to_mouse(0xF4) // report movement
}

/// A mouse packet: buttons and sign bits, then x and y movement.
struct Mouse {
    packet: [u8; 3],
    have: usize,
}

impl Mouse {
    /// Takes one byte; reports the move when a packet is complete.
    fn byte(&mut self, b: u8) -> bool {
        // The first byte always has bit 3 set: resynchronise on it.
        if self.have == 0 && b & 0x08 == 0 {
            return false;
        }
        self.packet[self.have] = b;
        self.have += 1;
        if self.have < 3 {
            return false;
        }
        self.have = 0;
        let [flags, x, y] = self.packet;
        let dx = x as i64 - if flags & 0x10 != 0 { 256 } else { 0 };
        let dy = y as i64 - if flags & 0x20 != 0 { 256 } else { 0 };
        // The mouse counts y upwards; the screen downwards.
        neo_rt::pointer_move(POINTER, dx, -dy, (flags & 7) as u64).is_ok()
    }
}

#[no_mangle]
fn neo_main() -> i64 {
    if init().is_err() {
        return -1;
    }
    let mouse = init_mouse().is_ok();
    let mut m = Mouse { packet: [0; 3], have: 0 };
    let (mut keys, mut typed) = (neo_keys::Keys::default(), 0i64);
    let mut extended = false;
    loop {
        let woke = if mouse { neo_rt::irq_wait_either(IRQ, MOUSE_IRQ) } else { neo_rt::irq_wait(IRQ) };
        if woke.is_err() {
            return typed;
        }
        // One interrupt can stand for more than one byte.
        while let Ok(status) = neo_rt::io_in(PORTS, STATUS) {
            if status & OUTPUT_FULL == 0 {
                break;
            }
            let Ok(code) = neo_rt::io_in(PORTS, DATA) else { break };
            if status & FROM_MOUSE != 0 {
                if mouse {
                    m.byte(code);
                }
                continue;
            }
            // 0xE0 says the next code is an arrow, Home, End or Delete.
            if code == 0xE0 {
                extended = true;
                continue;
            }
            // Set 1: bit 7 is the release; the rest is the key's number.
            let key = match core::mem::take(&mut extended) {
                true => neo_keys::ps2_extended(code & 0x7F),
                false => Some((code & 0x7F) as u16),
            };
            let Some(key) = key else { continue };
            // One key can type more than one byte (č is two, an arrow three).
            // The layout is the kernel's (the Bar can switch it); tell it
            // when Alt+Shift switched it here.
            let now = neo_rt::keyboard_layout(TYPING, neo_rt::LAYOUT_ASK).unwrap_or(0);
            keys.layout = neo_keys::Layout::from_number(now);
            let mut out = [0; 4];
            let n = keys.key_bytes(key, code & 0x80 == 0, &mut out);
            if keys.layout.number() != now {
                let _ = neo_rt::keyboard_layout(TYPING, keys.layout.number());
            }
            for &b in &out[..n] {
                if neo_rt::type_byte(TYPING, b).is_ok() {
                    typed += 1;
                }
            }
        }
    }
}
