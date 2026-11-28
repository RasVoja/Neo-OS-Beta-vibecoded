//! neo-rt: what a Fortress program needs to run.
//!
//! - `_start`, which aligns the stack and calls the program's `neo_main`;
//! - safe wrappers for every system call (numbers as in fortress/src/syscall.rs);
//! - `print!` / `println!` through the kernel's print call;
//! - a panic handler that prints the message and exits with code -1.
//!
//! With the `genode` feature the same program runs as a Genode component
//! instead (decision 0001): every system call goes to one C function,
//! `neo_genode_call`, which the component in `genode/` implements with
//! Genode's services, and there is no `_start` (Genode starts the
//! component, which calls `neo_genode_main`).
//!
//! A program looks like this:
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//! use neo_rt::println;
//!
//! #[no_mangle]
//! fn neo_main() -> i64 {
//!     println!("Hello!");
//!     0
//! }
//! ```

#![no_std]

#[cfg(not(feature = "genode"))]
use core::arch::{asm, global_asm};
use core::fmt;

pub type Handle = u64;

pub const ERR_BASE: u64 = u64::MAX - 0xFF;
pub const ERR_UNKNOWN_CALL: u64 = ERR_BASE;
pub const ERR_BAD_POINTER: u64 = ERR_BASE + 1;
pub const ERR_NOT_FOUND: u64 = ERR_BASE + 2;
pub const ERR_BAD_HANDLE: u64 = ERR_BASE + 3;
pub const ERR_DENIED: u64 = ERR_BASE + 4;
/// The disk has no room for the file, or the file is too large.
pub const ERR_FULL: u64 = ERR_BASE + 5;
pub const NO_HANDLE: Handle = u64::MAX;

/// Why a system call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnknownCall,
    BadPointer,
    NotFound,
    BadHandle,
    Denied,
    Other(u64),
}

fn check(r: u64) -> Result<u64, Error> {
    match r {
        ERR_UNKNOWN_CALL => Err(Error::UnknownCall),
        ERR_BAD_POINTER => Err(Error::BadPointer),
        ERR_NOT_FOUND => Err(Error::NotFound),
        ERR_BAD_HANDLE => Err(Error::BadHandle),
        ERR_DENIED => Err(Error::Denied),
        r if r >= ERR_BASE => Err(Error::Other(r)),
        r => Ok(r),
    }
}

#[cfg(feature = "genode")]
extern "C" {
    fn neo_genode_call(n: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64;
}

#[cfg(feature = "genode")]
#[inline(always)]
unsafe fn syscall(n: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    neo_genode_call(n, a0, a1, a2, a3)
}

#[cfg(all(target_arch = "x86_64", not(feature = "genode")))]
#[inline(always)]
unsafe fn syscall(n: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let r: u64;
    asm!(
        "syscall",
        inlateout("rax") n => r,
        inlateout("rdi") a0 => _,
        inlateout("rsi") a1 => _,
        inlateout("rdx") a2 => _,
        inlateout("r10") a3 => _,
        lateout("rcx") _, lateout("r11") _, lateout("r8") _, lateout("r9") _,
        options(nostack)
    );
    r
}

#[cfg(all(target_arch = "aarch64", not(feature = "genode")))]
#[inline(always)]
unsafe fn syscall(n: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let r: u64;
    asm!(
        "svc #0",
        in("x8") n,
        inlateout("x0") a0 => r,
        in("x1") a1,
        in("x2") a2,
        in("x3") a3,
        options(nostack)
    );
    r
}

/// Ends the program.
pub fn exit(code: i64) -> ! {
    unsafe { syscall(0, code as u64, 0, 0, 0) };
    loop {}
}

pub fn print(s: &str) -> Result<u64, Error> {
    check(unsafe { syscall(1, s.as_ptr() as u64, s.len() as u64, 0, 0) })
}

/// Looks a port up by name; needs the lookup permission.
pub fn find_port(name: &str) -> Result<Handle, Error> {
    check(unsafe { syscall(2, name.as_ptr() as u64, name.len() as u64, 0, 0) })
}

pub fn send(h: Handle, tag: u64, a0: u64, a1: u64) -> Result<(), Error> {
    check(unsafe { syscall(3, h, tag, a0, a1) }).map(|_| ())
}

pub fn sleep(ticks: u64) {
    unsafe { syscall(4, ticks, 0, 0, 0) };
}

/// A received message.
#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub tag: u64,
    pub a0: u64,
    pub a1: u64,
    pub sender: u64,
    /// Where to send the answer, if the sender wants one.
    pub reply: Option<Handle>,
    /// A handle that came with the message (shared memory, say).
    pub handle: Option<Handle>,
}

pub fn recv(h: Handle) -> Result<Message, Error> {
    recv_with(h, 0)
}

/// Like `recv`, but returns `None` at once when no message is waiting.
pub fn try_recv(h: Handle) -> Result<Option<Message>, Error> {
    match recv_with(h, 1) {
        Ok(m) => Ok(Some(m)),
        Err(Error::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

fn recv_with(h: Handle, poll: u64) -> Result<Message, Error> {
    let mut buf = [0u64; 6];
    check(unsafe { syscall(5, h, buf.as_mut_ptr() as u64, poll, 0) })?;
    let handle = |w: u64| (w != NO_HANDLE).then_some(w);
    Ok(Message { tag: buf[0], a0: buf[1], a1: buf[2], sender: buf[3], reply: handle(buf[4]), handle: handle(buf[5]) })
}

/// Sends a request and waits for the answer's first word.
pub fn call(h: Handle, tag: u64, a0: u64, a1: u64) -> Result<u64, Error> {
    check(unsafe { syscall(6, h, tag, a0, a1) })
}

pub fn close(h: Handle) -> Result<(), Error> {
    check(unsafe { syscall(7, h, 0, 0, 0) }).map(|_| ())
}

/// Writes a byte to an I/O port range (x86_64 only) the program was given (an I/O handle).
pub fn io_out(h: Handle, offset: u16, val: u8) -> Result<(), Error> {
    check(unsafe { syscall(8, h, offset as u64, val as u64, 0) }).map(|_| ())
}

/// Reads a byte from an I/O port range the program was given.
pub fn io_in(h: Handle, offset: u16) -> Result<u8, Error> {
    check(unsafe { syscall(9, h, offset as u64, 0, 0) }).map(|v| v as u8)
}

/// 16-bit port access; the two bytes must both be inside the range.
pub fn io_out16(h: Handle, offset: u16, val: u16) -> Result<(), Error> {
    check(unsafe { syscall(8, h, offset as u64, val as u64, 2) }).map(|_| ())
}

pub fn io_in16(h: Handle, offset: u16) -> Result<u16, Error> {
    check(unsafe { syscall(9, h, offset as u64, 0, 2) }).map(|v| v as u16)
}

/// Maps the device memory behind `mem` into this program; returns its
/// address.
pub fn map_device(mem: Handle) -> Result<*mut u8, Error> {
    check(unsafe { syscall(19, mem, 0, 0, 0) }).map(|a| a as *mut u8)
}

/// Opens a file of the boot image through a directory handle.
pub fn open(dir: Handle, name: &str) -> Result<Handle, Error> {
    check(unsafe { syscall(10, dir, name.as_ptr() as u64, name.len() as u64, 0) })
}

/// Reads from `offset` into `buf`; returns how many bytes were read.
pub fn read(file: Handle, buf: &mut [u8], offset: u64) -> Result<usize, Error> {
    check(unsafe { syscall(11, file, buf.as_mut_ptr() as u64, buf.len() as u64, offset) }).map(|n| n as usize)
}

/// Name of file number `index` in the directory, written into `buf`;
/// returns the full name length (the name may have been cut), or NotFound
/// past the last file.
pub fn list(dir: Handle, index: u64, buf: &mut [u8]) -> Result<usize, Error> {
    check(unsafe { syscall(12, dir, index, buf.as_mut_ptr() as u64, buf.len() as u64) }).map(|n| n as usize)
}

/// Writes the file `name` on the disk behind `dir` (replacing an earlier
/// one): `buf` holds the name and then the contents.
pub fn put(dir: Handle, buf: &[u8], name_len: usize) -> Result<usize, Error> {
    check(unsafe { syscall(40, dir, buf.as_ptr() as u64, buf.len() as u64, name_len as u64) }).map(|n| n as usize)
}

/// Deletes the file `name` on the disk behind `dir`.
pub fn remove(dir: Handle, name: &str) -> Result<(), Error> {
    check(unsafe { syscall(41, dir, name.as_ptr() as u64, name.len() as u64, 0) }).map(|_| ())
}

/// Whether the volume behind `dir` can be written, and its free bytes.
pub fn volume_status(dir: Handle) -> Result<(bool, u64), Error> {
    check(unsafe { syscall(42, dir, 0, 0, 0) }).map(|r| (r & 1 == 1, r >> 1))
}

/// The screen's width, height and bytes per line, asked through the
/// handle of its memory.
pub fn screen_mode(fb: Handle) -> Result<(u64, u64, u64), Error> {
    let mut v = [0u64; 3];
    check(unsafe { syscall(44, fb, v.as_mut_ptr() as u64, 0, 0) })?;
    Ok((v[0], v[1], v[2]))
}

/// Starts the program in `file`, giving it a copy of `pass` (or nothing,
/// with NO_HANDLE) as its handle 0. Returns its id.
pub fn spawn(file: Handle, pass: Handle) -> Result<u64, Error> {
    check(unsafe { syscall(13, file, pass, 0, 0) })
}

/// Exit code the kernel reports for a program it had to stop.
pub const STOPPED_CODE: i64 = -255;

/// Waits for a program this one started; returns its exit code.
pub fn wait(id: u64) -> Result<i64, Error> {
    // Exit codes can be negative, so they overlap the error range; the
    // kernel only fails here for an id that is not ours.
    let r = unsafe { syscall(14, id, 0, 0, 0) };
    if r == ERR_NOT_FOUND {
        Err(Error::NotFound)
    } else {
        Ok(r as i64)
    }
}

pub fn file_size(file: Handle) -> Result<u64, Error> {
    check(unsafe { syscall(15, file, 0, 0, 0) })
}

/// Waits until something is typed on the console (needs the console
/// handle) and reads it: at least one byte.
pub fn read_input(console: Handle, buf: &mut [u8]) -> Result<usize, Error> {
    check(unsafe { syscall(16, console, buf.as_mut_ptr() as u64, buf.len() as u64, 0) }).map(|n| n as usize)
}

/// Reads the kernel's log (its boot messages and what came after), from
/// `offset`; needs the console handle. Returns 0 at the end.
pub fn read_log(console: Handle, buf: &mut [u8], offset: u64) -> Result<usize, Error> {
    check(unsafe { syscall(27, console, buf.as_mut_ptr() as u64, buf.len() as u64, offset) }).map(|n| n as usize)
}

/// What the machine has right now, as text, one device a line:
/// `HW_USB` or `HW_PCI`. Needs the console handle. Copies what fits in
/// `buf`; returns the full length.
pub fn hardware(console: Handle, which: u64, buf: &mut [u8]) -> Result<usize, Error> {
    check(unsafe { syscall(28, console, buf.as_mut_ptr() as u64, buf.len() as u64, which) }).map(|n| n as usize)
}
pub const HW_USB: u64 = 0;

/// Asks Neo DOS to end, as if ENDCLI were typed; the system then shuts
/// down. Needs the key focus handle (the display service has it).
pub fn shut_down(focus: Handle) -> Result<(), Error> {
    check(unsafe { syscall(43, focus, 0, 0, 0) }).map(|_| ())
}

/// How busy the machine is: timer ticks since boot and how many found it
/// idle, ticks a second, usable RAM and free RAM in 4 KiB frames, threads.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Stats {
    pub ticks: u64,
    pub idle_ticks: u64,
    pub hz: u64,
    pub ram_frames: u64,
    pub free_frames: u64,
    pub threads: u64,
}

pub fn stats() -> Result<Stats, Error> {
    let mut v = [0u64; 6];
    check(unsafe { syscall(29, v.as_mut_ptr() as u64, 0, 0, 0) })?;
    Ok(Stats { ticks: v[0], idle_ticks: v[1], hz: v[2], ram_frames: v[3], free_frames: v[4], threads: v[5] })
}
pub const HW_PCI: u64 = 1;

/// Waits until the interrupt line behind `irq` fires; returns how many
/// times it fired since the last wait. Fails when the kernel takes the
/// line back.
pub fn irq_wait(irq: Handle) -> Result<u64, Error> {
    check(unsafe { syscall(17, irq, 0, 0, 0) })
}

/// Waits until either of two interrupts has fired (a driver for a device
/// with two lines, like the PC's keyboard controller).
pub fn irq_wait_either(irq: Handle, other: Handle) -> Result<u64, Error> {
    check(unsafe { syscall(17, irq, other + 1, 0, 0) })
}

/// Moves the pointer by `dx`, `dy` screen pixels (y grows downwards) with
/// `buttons` down (bit 0 left, 1 right, 2 middle); needs a pointer handle.
pub fn pointer_move(pointer: Handle, dx: i64, dy: i64, buttons: u64) -> Result<(), Error> {
    check(unsafe { syscall(20, pointer, dx as u64, dy as u64, buttons) }).map(|_| ())
}

/// Makes `pages` pages of zeroed memory to share with another program;
/// map it with `map_device` and pass it with `send_handle`.
pub fn share(pages: u64) -> Result<Handle, Error> {
    check(unsafe { syscall(21, pages, 0, 0, 0) })
}

/// Sends a message that carries a copy of `handle`; the receiver finds it
/// in `Message::handle`.
pub fn send_handle(port: Handle, tag: u64, a0: u64, handle: Handle) -> Result<(), Error> {
    check(unsafe { syscall(22, port, tag, a0, handle) }).map(|_| ())
}

/// Makes a new message port with room for `capacity` messages (1-64);
/// the handle has both rights. Pass it with `send_handle` to let another
/// program send to it.
pub fn new_port(capacity: u64) -> Result<Handle, Error> {
    check(unsafe { syscall(23, capacity, 0, 0, 0) })
}

/// Like `send`, but never waits: returns false if the port was full and
/// the message was dropped.
pub fn try_send(h: Handle, tag: u64, a0: u64, a1: u64) -> Result<bool, Error> {
    check(unsafe { syscall(24, h, tag, a0, a1) }).map(|r| r == 0)
}

/// The display service's protocol. `WINDOW_OPEN` carries the window's
/// pixels as shared memory (width x height 32-bit pixels, 0xRRGGBB), with
/// `a0` = width << 16 | height and the title's first 8 bytes packed in a
/// second message `WINDOW_TITLE` before it. `WINDOW_DRAWN` asks for the
/// window to be shown again after the program drew into it. A program has
/// one window; the display knows it by the sender.
pub const WINDOW_TITLE: u64 = 4;
pub const WINDOW_OPEN: u64 = 5;
pub const WINDOW_DRAWN: u64 = 6;
/// Before `WINDOW_OPEN`: the handle that comes with it is a port where the
/// window's events go (the service may only send to it).
pub const WINDOW_EVENTS: u64 = 7;
/// The program closes its window.
pub const WINDOW_CLOSE: u64 = 8;
/// An event for a window: a click in it. `a0` is x << 16 | y inside the
/// window's pixels, `a1` the buttons just pressed (bit 0 left, 1 right).
pub const EVENT_CLICK: u64 = 16;
/// An event for a window: the display service is stopping; the program
/// should end.
pub const EVENT_QUIT: u64 = 17;
/// An event for a window: a key typed while it has the focus; `a0` is the
/// byte.
pub const EVENT_KEY: u64 = 18;

/// Sends what the keyboards type to `port` (a window's event port), or back
/// to the console with `None`. Needs the key focus handle the display
/// service gets.
pub fn focus_keys(focus: Handle, port: Option<Handle>) -> Result<(), Error> {
    check(unsafe { syscall(25, focus, port.unwrap_or(NO_HANDLE), 0, 0) }).map(|_| ())
}

/// The keyboard layout (0 US, 1 Serbian Latin), shared by the keyboard
/// drivers and the Bar. `op` is 0 or 1 to set it, `LAYOUT_SWITCH` to switch,
/// `LAYOUT_ASK` to only read it; returns the layout after. Needs a typing or
/// a key focus handle.
pub fn keyboard_layout(h: Handle, op: u64) -> Result<u64, Error> {
    check(unsafe { syscall(26, h, op, 0, 0) })
}
pub const LAYOUT_SWITCH: u64 = 2;
pub const LAYOUT_ASK: u64 = 3;
/// The display service hears about a new layout with this tag; a0 is it.
pub const LAYOUT_CHANGED: u64 = 9;

/// Asks the display service for one of Dijana's tips after an error, as
/// the specification (8c) says: she speaks up by herself only at first
/// start, after an error and when called. `a0` is one of the `TIP_` kinds.
pub const DIJANA_TIP: u64 = 10;
pub const TIP_UNKNOWN_COMMAND: u64 = 1;
pub const TIP_NO_FILE: u64 = 2;
pub const TIP_PROGRAM_STOPPED: u64 = 3;
pub const TIP_PROGRAM_FAILED: u64 = 4;
/// `a0` of the display service's QUIT when the system is shutting down:
/// Dijana says good night and curls up before the desk goes.
pub const QUIT_GOOD_NIGHT: u64 = 1;

/// Sends one of Dijana's tips to the display service, if there is one and
/// the program may look ports up; never waits.
pub fn dijana_tip(kind: u64) {
    if let Ok(display) = find_port("display") {
        let _ = try_send(display, DIJANA_TIP, kind, 0);
        let _ = close(display);
    }
}

/// The tag of the pointer messages the display service gets: `a0` is x,
/// `a1` is y plus the buttons times 65536.
pub const POINTER_MOVED: u64 = 3;

/// Adds one byte straight to what the console reads, even while a window
/// has the keyboard focus (a terminal passing on what is typed in it).
pub fn type_to_console(typing: Handle, b: u8) -> Result<(), Error> {
    check(unsafe { syscall(18, typing, b as u64, 1, 0) }).map(|_| ())
}

/// Adds one byte to what the console reads (needs a typing handle); while
/// a window has the keyboard focus, it goes to that window instead.
pub fn type_byte(typing: Handle, b: u8) -> Result<(), Error> {
    check(unsafe { syscall(18, typing, b as u64, 0, 0) }).map(|_| ())
}

/// The console service's protocol: tag `CONSOLE_TEXT | len << 8` carries up
/// to 16 bytes of text packed little-endian in a0/a1; `CONSOLE_QUIT` stops it.
pub const CONSOLE_TEXT: u64 = 1;
pub const CONSOLE_QUIT: u64 = 2;

/// Sends text to the console service through a send handle.
pub fn console_write(h: Handle, s: &str) -> Result<(), Error> {
    for chunk in s.as_bytes().chunks(16) {
        let mut buf = [0u8; 16];
        buf[..chunk.len()].copy_from_slice(chunk);
        let a0 = u64::from_le_bytes(buf[..8].try_into().unwrap());
        let a1 = u64::from_le_bytes(buf[8..].try_into().unwrap());
        send(h, CONSOLE_TEXT | (chunk.len() as u64) << 8, a0, a1)?;
    }
    Ok(())
}

/// A `fmt::Write` that goes to the console service.
pub struct ConsoleClient(pub Handle);

impl fmt::Write for ConsoleClient {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        console_write(self.0, s).map_err(|_| fmt::Error)
    }
}

pub struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s).map(|_| ()).map_err(|_| fmt::Error)
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = fmt::Write::write_fmt(&mut Console, args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{ $crate::_print(format_args!($($arg)*)); $crate::print!("\n"); }};
}

// The kernel starts a program with the stack pointer at the top of its
// stack; align it the way the ABI expects before calling Rust.
#[cfg(all(target_arch = "x86_64", not(feature = "genode")))]
global_asm!(
    ".pushsection .text._start, \"ax\"",
    ".global _start",
    "_start:",
    "    and rsp, -16",
    "    xor ebp, ebp",
    "    call {start}",
    "    ud2",
    ".popsection",
    start = sym rust_start,
);

#[cfg(all(target_arch = "aarch64", not(feature = "genode")))]
global_asm!(
    ".pushsection .text._start, \"ax\"",
    ".global _start",
    "_start:",
    "    mov x9, sp",
    "    and sp, x9, #-16",
    "    mov x29, #0",
    "    mov x30, #0",
    "    bl {start}",
    "    brk #1",
    ".popsection",
    start = sym rust_start,
);

extern "Rust" {
    fn neo_main() -> i64;
}

#[cfg(not(feature = "genode"))]
extern "C" fn rust_start() -> ! {
    exit(unsafe { neo_main() })
}

/// On Genode the component calls this; it returns the program's exit code.
#[cfg(feature = "genode")]
#[no_mangle]
pub extern "C" fn neo_genode_main() -> i64 {
    unsafe { neo_main() }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info.message());
    exit(-1)
}
