//! System calls: the only way a program reaches the kernel.
//!
//! | nr  | call      | arguments                       | result                 |
//! |-----|-----------|---------------------------------|------------------------|
//! | 0   | exit      | code                            | does not return        |
//! | 1   | print     | text ptr, length                | bytes written          |
//! | 2   | find_port | name ptr, length                | handle (send right)    |
//! | 3   | send      | handle, tag, a0, a1             | 0                      |
//! | 4   | sleep     | ticks                           | 0                      |
//! | 5   | recv      | handle, buffer ptr (6 words), poll | 0                   |
//! | 6   | call      | handle, tag, a0, a1             | answer's first word    |
//! | 7   | close     | handle                          | 0                      |
//! | 8   | io_out    | I/O handle, offset, value, width| 0                      |
//! | 9   | io_in     | I/O handle, offset, -, width    | value                  |
//! | 10  | open      | dir handle, name ptr, length    | file handle            |
//! | 11  | read      | file handle, buf, length, offset| bytes read             |
//! | 12  | list      | dir handle, index, buf, length  | name length            |
//! | 13  | spawn     | file handle, handle to pass     | program id             |
//! | 14  | wait      | program id                      | its exit code          |
//! | 15  | file_size | file or shared memory handle    | size in bytes          |
//! | 16  | read_input| console handle, buf, length     | bytes read             |
//! | 17  | irq_wait  | interrupt handle, other + 1 or 0| times it fired         |
//! | 18  | type_byte | typing handle, byte, to console | 0                      |
//! | 19  | map_device| memory handle                   | address it is mapped at|
//! | 20  | pointer_move | pointer handle, dx, dy, buttons | 0                   |
//! | 21  | share     | pages                           | shared memory handle   |
//! | 22  | send_handle | port handle, tag, a0, handle  | 0                      |
//! | 23  | new_port  | capacity (1-64)                 | port handle (both rights) |
//! | 24  | try_send  | handle, tag, a0, a1             | 0 sent, 1 port full    |
//! | 25  | focus_keys| key focus handle, port or NO_HANDLE | 0                  |
//! | 26  | keyboard_layout | typing or key focus handle, 0/1 set, 2 switch, 3 ask | layout (0 US, 1 SR) |
//! | 27  | read_log  | console handle, buffer, length, offset | bytes read        |
//! | 28  | hardware  | console handle, buffer, length, 0 USB / 1 PCI | full text length |
//! | 29  | stats     | buffer of 6 u64                 | 0                      |
//! | 40  | put       | disk directory handle, buffer, total length, name length | 0 |
//! | 41  | remove    | disk directory handle, name ptr, length | 0              |
//! | 42  | volume_status | directory handle            | free bytes << 1, bit 0: writable |
//! | 43  | shut_down | key focus handle                | 0                      |
//!
//! `shut_down` asks the console's program (Neo DOS) to end, as if ENDCLI
//! were typed: it puts an end-of-input byte (0x04) in the console input,
//! whoever has the key focus. When Neo DOS ends, the system shuts down.
//!
//! `stats` fills: timer ticks, of them idle, the timer rate (ticks a
//! second), usable RAM frames, free frames, threads. Anyone may ask.
//!
//! On x86_64 the number goes in rax and the arguments in rdi, rsi, rdx, r10
//! (`syscall`); on AArch64 the number goes in x8 and the arguments in x0-x3
//! (`svc #0`). The result comes back in rax or x0. Calls 8 and 9 exist on
//! x86_64 only. `recv` with poll set to 1 returns NOT_FOUND at once when
//! the port is empty. `io_out` and `io_in` take a width of 1, 2 or 4 bytes
//! (0 means 1). `irq_wait` waits on a second interrupt handle too when its
//! second argument is that handle plus one. `map_device` maps the memory
//! behind the handle into the program, in the window at `DEVICE_BASE`.
//!
//! `recv` fills the buffer with tag, a0, a1, sender, reply handle (or
//! `NO_HANDLE`) and a passed handle (or `NO_HANDLE`): `send_handle` sends a
//! copy of one of the sender's handles with the message (a1 is then the
//! sender's handle number, for information); a port passed this way keeps
//! only its send right. `try_send` is `send` that never waits: a full port
//! drops the message and the call returns 1. `share` makes zeroed shared
//! memory, up to 1024 pages, which `map_device` maps. A reply handle has the send right only. `find_port` needs
//! the lookup permission; `send` and `call` need the send right on the
//! handle, `recv` the receive right. `io_out` and `io_in` need an I/O handle
//! and an offset inside its range.
//!
//! Files: a directory handle (a volume: the boot image or the disk) lets a
//! program list files and open them; a file handle lets it read the file and start it as a new
//! program. `list` writes the name of file number `index` into the buffer
//! (cut to its length) and returns the full name length, or NOT_FOUND past
//! the last file. `spawn` gives the new program a copy of the handle to pass
//! (or none, with NO_HANDLE) as its handle 0; `wait` works only on programs
//! the caller started, and returns -255 for a program that was stopped.
//!
//! `read_input` needs the console handle; it waits until something is
//! typed and returns what is there, at least one byte.
//!
//! Errors are returned as values from `ERR_BASE` up. Every pointer a
//! program passes is checked against its own address space first.

use crate::process::{self, Cap, FileData, Kind, Volume, RIGHT_RECV, RIGHT_SEND};
use crate::{bootfs, diskfs, ipc, sched};

pub const ERR_BASE: u64 = u64::MAX - 0xFF;
pub const ERR_UNKNOWN_CALL: u64 = ERR_BASE;
pub const ERR_BAD_POINTER: u64 = ERR_BASE + 1;
pub const ERR_NOT_FOUND: u64 = ERR_BASE + 2;
pub const ERR_BAD_HANDLE: u64 = ERR_BASE + 3;
pub const ERR_DENIED: u64 = ERR_BASE + 4;

pub const NO_HANDLE: u64 = u64::MAX;

const MAX_TEXT: u64 = 4096;

fn range_ok(ptr: u64, len: u64, write: bool) -> bool {
    sched::with_current_process(|p| p.space().user_range_ok(ptr, len, write)).unwrap_or(false)
}

/// A program's bytes, after checking they are its own, mapped memory.
fn user_bytes(ptr: u64, len: u64) -> Option<&'static [u8]> {
    // Safe to read: the range is mapped for this program in the active
    // address space, and nothing else runs in it while we do.
    (len <= MAX_TEXT && range_ok(ptr, len, false))
        .then(|| unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) })
}

/// The port behind a handle of the calling program, if it has `right`.
fn port(handle: u64, right: u8) -> Result<alloc::sync::Arc<ipc::Port>, u64> {
    let found = sched::with_current_process(|p| (p.kind(handle).is_some(), p.port(handle, right)));
    match found {
        Some((_, Some(port))) => Ok(port),
        Some((true, None)) => Err(ERR_DENIED),
        _ => Err(ERR_BAD_HANDLE),
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let r = match num {
        0 => process::exit(a0 as i64),
        1 => print(a0, a1),
        2 => find_port(a0, a1),
        3 => port(a0, RIGHT_SEND).map(|p| {
            p.send(ipc::Message::new(a1, [a2, a3, sched::current_id()]));
            0
        }),
        4 => {
            sched::sleep(a0.min(1000));
            Ok(0)
        }
        5 => recv(a0, a1, a2 == 1),
        6 => port(a0, RIGHT_SEND).map(|p| p.call(ipc::Message::new(a1, [a2, a3, sched::current_id()])).args[0]),
        7 => match sched::with_current_process(|p| p.close(a0)) {
            Some(true) => Ok(0),
            _ => Err(ERR_BAD_HANDLE),
        },
        #[cfg(target_arch = "x86_64")]
        8 | 9 => {
            // a3: width in bytes (0 or 1, 2 or 4); the whole access must
            // fall inside the range.
            let width = a3.max(1);
            if ![1, 2, 4].contains(&width) {
                return ERR_DENIED;
            }
            let found = sched::with_current_process(|p| {
                (p.kind(a0).is_some(), p.io_port(a0, a1), p.io_port(a0, a1 + width - 1))
            });
            match found {
                Some((_, Some(io), Some(_))) => {
                    if num == 8 {
                        crate::arch::io_out(io, a2 as u32, width);
                        Ok(0)
                    } else {
                        Ok(crate::arch::io_in(io, width) as u64)
                    }
                }
                Some((true, _, _)) => Err(ERR_DENIED),
                _ => Err(ERR_BAD_HANDLE),
            }
        }
        10 => open(a0, a1, a2),
        11 => read(a0, a1, a2, a3),
        12 => list(a0, a1, a2, a3),
        13 => spawn(a0, a1),
        14 => match sched::with_current_process(|p| p.is_child(a0)) {
            Some(true) => Ok(process::wait(a0) as u64),
            _ => Err(ERR_NOT_FOUND),
        },
        15 => match sched::with_current_process(|p| p.shared_bytes(a0)).flatten() {
            Some(bytes) => Ok(bytes),
            None => file(a0).map(|(_, data)| data.bytes().len() as u64),
        },
        16 => read_input(a0, a1, a2),
        17 => irq_wait(a0, a1),
        19 => match sched::with_current_process(|p| (p.kind(a0), p.map_device(a0))) {
            Some((_, Some(at))) => Ok(at),
            Some((Some(_), None)) => Err(ERR_DENIED),
            _ => Err(ERR_BAD_HANDLE),
        },
        21 => share(a0),
        22 => send_handle(a0, a1, a2, a3),
        23 => match a0 {
            1..=64 => {
                sched::with_current_process(|p| p.grant(Cap::new(ipc::Port::new(a0 as usize), RIGHT_SEND | RIGHT_RECV)))
                    .ok_or(ERR_BAD_HANDLE)
            }
            _ => Err(ERR_DENIED),
        },
        25 => focus_keys(a0, a1),
        26 => keyboard_layout(a0, a1),
        24 => port(a0, RIGHT_SEND).map(|p| !p.try_send(ipc::Message::new(a1, [a2, a3, sched::current_id()])) as u64),
        20 => match sched::with_current_process(|p| p.kind(a0)).flatten() {
            Some(Kind::Pointer) => {
                crate::pointer::moved(a1 as i64, a2 as i64, a3);
                Ok(0)
            }
            Some(_) => Err(ERR_DENIED),
            None => Err(ERR_BAD_HANDLE),
        },
        18 => match sched::with_current_process(|p| p.kind(a0)).flatten() {
            Some(Kind::Typing) => {
                // a2 = 1: straight into the console (a terminal passing on
                // keys), not to the window in focus.
                if a2 == 1 {
                    crate::input::push_console(a1 as u8);
                } else {
                    crate::input::push(a1 as u8);
                }
                Ok(0)
            }
            Some(_) => Err(ERR_DENIED),
            None => Err(ERR_BAD_HANDLE),
        },
        27 => read_log(a0, a1, a2, a3),
        28 => hardware(a0, a1, a2, a3),
        29 => stats(a0),
        40 => put(a0, a1, a2, a3),
        41 => remove(a0, a1, a2),
        42 => volume_status(a0),
        44 => screen_mode(a0, a1),
        45 => Ok(crate::clock::unix_seconds()),
        43 => match sched::with_current_process(|p| p.kind(a0)).flatten() {
            Some(Kind::KeyFocus) => {
                crate::input::push_console(0x04);
                Ok(0)
            }
            Some(_) => Err(ERR_DENIED),
            None => Err(ERR_BAD_HANDLE),
        },
        _ => Err(ERR_UNKNOWN_CALL),
    };
    r.unwrap_or_else(|e| e)
}

/// Longest read or name a single call handles.
const MAX_READ: u64 = 64 * 1024;

/// The volume behind the calling program's directory `handle`.
fn dir(handle: u64) -> Result<Volume, u64> {
    match sched::with_current_process(|p| (p.kind(handle), p.volume(handle))) {
        Some((_, Some(v))) => Ok(v),
        Some((Some(_), None)) => Err(ERR_DENIED),
        _ => Err(ERR_BAD_HANDLE),
    }
}

fn file(handle: u64) -> Result<(&'static str, FileData), u64> {
    match sched::with_current_process(|p| (p.kind(handle), p.file(handle))) {
        Some((_, Some(f))) => Ok(f),
        Some((Some(_), None)) => Err(ERR_DENIED),
        _ => Err(ERR_BAD_HANDLE),
    }
}

/// Program names must live as long as the kernel (thread names); the same
/// name is kept once, however often it is opened.
fn intern(name: &str) -> &'static str {
    static NAMES: crate::sync::SpinLock<alloc::vec::Vec<&'static str>> =
        crate::sync::SpinLock::new(alloc::vec::Vec::new());
    let mut names = NAMES.lock();
    if let Some(n) = names.iter().find(|n| **n == name) {
        return n;
    }
    let n: &'static str = alloc::boxed::Box::leak(alloc::string::String::from(name).into_boxed_str());
    names.push(n);
    n
}

fn base_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn open(handle: u64, ptr: u64, len: u64) -> Result<u64, u64> {
    let volume = dir(handle)?;
    let name = user_bytes(ptr, len).and_then(|b| core::str::from_utf8(b).ok()).ok_or(ERR_BAD_POINTER)?;
    let cap = match volume {
        Volume::Boot => {
            let fs = bootfs::image().ok_or(ERR_NOT_FOUND)?;
            let f = fs.files().find(|f| f.name() == name).ok_or(ERR_NOT_FOUND)?;
            Cap::File { name: f.base_name(), data: FileData::Static(f.data) }
        }
        Volume::Disk => {
            let data = diskfs::read_file(name).map_err(|_| ERR_NOT_FOUND)?;
            Cap::File { name: intern(base_name(name)), data: FileData::Owned(data.into()) }
        }
    };
    sched::with_current_process(|p| p.grant(cap)).ok_or(ERR_BAD_HANDLE)
}

/// Most a single `put` takes: the name and the contents together.
const MAX_PUT: u64 = 256 * 1024;

/// The disk has no room for it, or the file is too large.
pub const ERR_FULL: u64 = ERR_BASE + 5;

fn disk_error(e: &str) -> u64 {
    match e {
        "no such file" => ERR_NOT_FOUND,
        "bin/ is kept as it is" | "the disk is read-only" => ERR_DENIED,
        "the disk is full" | "file too large" => ERR_FULL,
        _ => ERR_BAD_POINTER,
    }
}

/// Writes a file to the disk behind directory `handle`: `buf` holds the
/// name (`name_len` bytes) followed by the contents. The boot image is
/// read only.
fn put(handle: u64, buf: u64, len: u64, name_len: u64) -> Result<u64, u64> {
    let volume = dir(handle)?;
    if name_len > len || len > MAX_PUT || !range_ok(buf, len, false) {
        return Err(ERR_BAD_POINTER);
    }
    // Checked above: mapped, readable and the program's own.
    let all = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) };
    let (name, data) = all.split_at(name_len as usize);
    let name = core::str::from_utf8(name).map_err(|_| ERR_BAD_POINTER)?;
    match volume {
        Volume::Boot => Err(ERR_DENIED),
        Volume::Disk => diskfs::write_file(name, data).map(|_| data.len() as u64).map_err(disk_error),
    }
}

/// Deletes a file of the disk behind directory `handle`.
fn remove(handle: u64, ptr: u64, len: u64) -> Result<u64, u64> {
    let volume = dir(handle)?;
    let name = user_bytes(ptr, len).and_then(|b| core::str::from_utf8(b).ok()).ok_or(ERR_BAD_POINTER)?;
    match volume {
        Volume::Boot => Err(ERR_DENIED),
        Volume::Disk => diskfs::delete_file(name).map(|_| 0).map_err(disk_error),
    }
}

/// What a volume allows: bit 0 set when it can be written, the rest is the
/// free space in bytes (0 when read only).
fn volume_status(handle: u64) -> Result<u64, u64> {
    match dir(handle)? {
        Volume::Boot => Ok(0),
        Volume::Disk => {
            if crate::virtio_blk::writable() {
                Ok(diskfs::free_bytes().unwrap_or(0) << 1 | 1)
            } else {
                Ok(0)
            }
        }
    }
}

/// The screen's mode for whoever holds its memory (`handle`): width,
/// height and bytes per line, as three words at `buf`.
fn screen_mode(handle: u64, buf: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::Mem) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    if buf % 8 != 0 || !range_ok(buf, 24, true) {
        return Err(ERR_BAD_POINTER);
    }
    let (w, h, pitch) = crate::display::mode();
    let values = [w, h, pitch];
    // Checked above: mapped, writable, aligned and the program's own.
    unsafe { core::ptr::copy_nonoverlapping(values.as_ptr(), buf as *mut u64, 3) };
    Ok(0)
}

fn read(handle: u64, buf: u64, len: u64, offset: u64) -> Result<u64, u64> {
    let (_, data) = file(handle)?;
    let data = data.bytes();
    let len = len.min(MAX_READ);
    if !range_ok(buf, len, true) {
        return Err(ERR_BAD_POINTER);
    }
    let from = (offset.min(data.len() as u64)) as usize;
    let n = (data.len() - from).min(len as usize);
    // Checked above: mapped, writable and the program's own.
    unsafe { core::ptr::copy_nonoverlapping(data.as_ptr().add(from), buf as *mut u8, n) };
    Ok(n as u64)
}

fn list(handle: u64, index: u64, buf: u64, len: u64) -> Result<u64, u64> {
    let volume = dir(handle)?;
    let len = len.min(MAX_READ);
    if !range_ok(buf, len, true) {
        return Err(ERR_BAD_POINTER);
    }
    let copy = |name: &[u8]| {
        let n = name.len().min(len as usize);
        // Checked above: mapped, writable and the program's own.
        unsafe { core::ptr::copy_nonoverlapping(name.as_ptr(), buf as *mut u8, n) };
        name.len() as u64
    };
    match volume {
        Volume::Boot => {
            let fs = bootfs::image().ok_or(ERR_NOT_FOUND)?;
            let f = fs.files().nth(index as usize).ok_or(ERR_NOT_FOUND)?;
            Ok(copy(f.name().as_bytes()))
        }
        Volume::Disk => {
            let files = diskfs::list().map_err(|_| ERR_NOT_FOUND)?;
            let f = files.get(index as usize).ok_or(ERR_NOT_FOUND)?;
            Ok(copy(f.name.as_bytes()))
        }
    }
}

/// The kernel's log, for whoever holds the console (Neo DOS).
fn read_log(handle: u64, buf: u64, len: u64, offset: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::Console) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    let len = len.min(MAX_READ);
    if !range_ok(buf, len, true) {
        return Err(ERR_BAD_POINTER);
    }
    // Checked above: mapped, writable and the program's own.
    let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len as usize) };
    Ok(crate::console::read_log(offset as usize, out) as u64)
}

/// How busy the machine is, for the Meter tile.
fn stats(buf: u64) -> Result<u64, u64> {
    if buf % 8 != 0 || !range_ok(buf, 48, true) {
        return Err(ERR_BAD_POINTER);
    }
    use core::sync::atomic::Ordering::Relaxed;
    let values = [
        sched::TICKS.load(Relaxed),
        sched::IDLE_TICKS.load(Relaxed),
        crate::arch::HZ as u64,
        crate::mem::frame::usable_frames() as u64,
        crate::mem::frame::free_frames() as u64,
        sched::thread_count() as u64,
    ];
    // Checked above: mapped, writable, aligned and the program's own.
    unsafe { core::ptr::copy_nonoverlapping(values.as_ptr(), buf as *mut u64, values.len()) };
    Ok(0)
}

/// What the machine has right now, one device a line: the USB devices
/// (0) or the PCI functions (1). Copies what fits; returns the full length.
fn hardware(handle: u64, buf: u64, len: u64, which: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::Console) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    let len = len.min(MAX_READ);
    if !range_ok(buf, len, true) {
        return Err(ERR_BAD_POINTER);
    }
    #[cfg(target_arch = "x86_64")]
    let text = match which {
        0 => crate::usb::listing(),
        1 => crate::pci::listing(),
        _ => return Err(ERR_NOT_FOUND),
    };
    #[cfg(not(target_arch = "x86_64"))]
    let text = match which {
        0 | 1 => alloc::string::String::new(),
        _ => return Err(ERR_NOT_FOUND),
    };
    let n = text.len().min(len as usize);
    // Checked above: mapped, writable and the program's own.
    let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
    out.copy_from_slice(&text.as_bytes()[..n]);
    Ok(text.len() as u64)
}

fn read_input(handle: u64, buf: u64, len: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::Console) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    let len = len.min(MAX_READ);
    if len == 0 || !range_ok(buf, len, true) {
        return Err(ERR_BAD_POINTER);
    }
    // Checked above: mapped, writable and the program's own. The kernel
    // buffer keeps the program's memory out of the interrupt path.
    let mut tmp = [0u8; 256];
    let n = crate::input::read(&mut tmp[..len.min(256) as usize]);
    unsafe { core::ptr::copy_nonoverlapping(tmp.as_ptr(), buf as *mut u8, n) };
    Ok(n as u64)
}

/// Waits for the interrupt line behind `handle`; returns how many times
/// it fired. Fails once the kernel has taken the line back.
/// Waits on the interrupt behind `handle`, and also on the one behind
/// handle `also - 1` when `also` is not 0.
fn irq_wait(handle: u64, also: u64) -> Result<u64, u64> {
    let line = |h: u64| match sched::with_current_process(|p| (p.kind(h), p.irq_line(h))) {
        Some((_, Some(line))) => Ok(line),
        Some((Some(_), None)) => Err(ERR_DENIED),
        _ => Err(ERR_BAD_HANDLE),
    };
    let first = line(handle)?;
    let r = if also == 0 { crate::irq::wait(&[first]) } else { crate::irq::wait(&[first, line(also - 1)?]) };
    r.ok_or(ERR_DENIED)
}

/// Keyboard bytes go to the port behind `target` (a send right is enough),
/// or back to the console with `NO_HANDLE`. Needs the key focus capability.
fn focus_keys(handle: u64, target: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::KeyFocus) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    let port = if target == NO_HANDLE { None } else { Some(port(target, RIGHT_SEND)?) };
    crate::input::set_focus(port);
    Ok(0)
}

/// The keyboard layout: a keyboard driver (typing capability) sets it when
/// Alt+Shift switches it, the display service (key focus) when the Bar's
/// switch is clicked. 0 or 1 sets it, 2 switches it, 3 only asks.
fn keyboard_layout(handle: u64, op: u64) -> Result<u64, u64> {
    match sched::with_current_process(|p| p.kind(handle)).flatten() {
        Some(Kind::Typing | Kind::KeyFocus) => {}
        Some(_) => return Err(ERR_DENIED),
        None => return Err(ERR_BAD_HANDLE),
    }
    match op {
        0 | 1 => crate::input::set_layout(op),
        2 => crate::input::set_layout(crate::input::layout() ^ 1),
        3 => {}
        _ => return Err(ERR_DENIED),
    }
    Ok(crate::input::layout())
}

fn share(pages: u64) -> Result<u64, u64> {
    let sh = process::Shared::new(pages).ok_or(ERR_DENIED)?;
    sched::with_current_process(|p| p.grant(Cap::Shared(sh))).ok_or(ERR_BAD_HANDLE)
}

fn send_handle(handle: u64, tag: u64, a0: u64, pass: u64) -> Result<u64, u64> {
    let target = port(handle, RIGHT_SEND)?;
    let cap = match sched::with_current_process(|p| p.cap(pass)).flatten().ok_or(ERR_BAD_HANDLE)? {
        // The receiver may send to a passed port, not take its messages.
        Cap::Port { port, rights } => Cap::Port { port, rights: rights & RIGHT_SEND },
        cap => cap,
    };
    let mut m = ipc::Message::new(tag, [a0, pass, sched::current_id()]);
    m.cap = Some(cap);
    target.send(m);
    Ok(0)
}

fn spawn(file_handle: u64, pass: u64) -> Result<u64, u64> {
    let (name, data) = file(file_handle)?;
    let data = data.bytes();
    let caps = if pass == NO_HANDLE {
        alloc::vec![]
    } else {
        alloc::vec![sched::with_current_process(|p| p.cap(pass)).flatten().ok_or(ERR_BAD_HANDLE)?]
    };
    // A program started by another may look ports up if its parent may.
    let lookup = sched::with_current_process(|p| p.may_lookup).unwrap_or(false);
    let id = process::spawn_elf(name, data, caps, lookup).map_err(|_| ERR_DENIED)?;
    sched::with_current_process(|p| p.add_child(id));
    Ok(id)
}

fn print(ptr: u64, len: u64) -> Result<u64, u64> {
    let b = user_bytes(ptr, len).ok_or(ERR_BAD_POINTER)?;
    for &c in b {
        // Text, backspace, and UTF-8 bytes (for č, ž, ...) pass; other
        // control bytes show as '?'.
        crate::arch::putc(if c == b'\n' || c == 0x08 || (0x20..0x7F).contains(&c) || c >= 0x80 { c } else { b'?' });
    }
    crate::console::tee(b);
    Ok(len)
}

fn find_port(ptr: u64, len: u64) -> Result<u64, u64> {
    if !sched::with_current_process(|p| p.may_lookup).unwrap_or(false) {
        return Err(ERR_DENIED);
    }
    let name = user_bytes(ptr, len).and_then(|b| core::str::from_utf8(b).ok()).ok_or(ERR_BAD_POINTER)?;
    let port = ipc::find(name).ok_or(ERR_NOT_FOUND)?;
    sched::with_current_process(|p| p.grant(Cap::new(port, RIGHT_SEND))).ok_or(ERR_BAD_HANDLE)
}

/// Takes the next message from a port; with `poll`, returns
/// `ERR_NOT_FOUND` at once instead of waiting when there is none.
fn recv(handle: u64, buf: u64, poll: bool) -> Result<u64, u64> {
    if !range_ok(buf, 6 * 8, true) {
        return Err(ERR_BAD_POINTER);
    }
    let p = port(handle, RIGHT_RECV)?;
    let m = if poll { p.try_recv().ok_or(ERR_NOT_FOUND)? } else { p.recv() };
    let reply = match m.reply {
        Some(r) => sched::with_current_process(|p| p.grant(Cap::new(r, RIGHT_SEND))).unwrap_or(NO_HANDLE),
        None => NO_HANDLE,
    };
    let passed = match m.cap {
        Some(c) => sched::with_current_process(|p| p.grant(c)).unwrap_or(NO_HANDLE),
        None => NO_HANDLE,
    };
    let out = [m.tag, m.args[0], m.args[1], m.args[2], reply, passed];
    // Checked above: mapped, writable and the program's own.
    unsafe { core::ptr::copy_nonoverlapping(out.as_ptr(), buf as *mut u64, 6) };
    Ok(0)
}
