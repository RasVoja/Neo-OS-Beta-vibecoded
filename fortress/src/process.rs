//! Programs: code running in user mode, each in its own address space.
//!
//! A process here is one thread plus an address space plus a table of
//! capabilities. The program can only reach the kernel through system calls
//! (see syscall.rs); touching kernel memory or running a privileged
//! instruction stops the program, and the rest of the system carries on.
//!
//! Capabilities (the Gate idea): a program can use a port only through a
//! handle in its own table, and each handle carries rights (send, receive).
//! A handle can also stand for a range of I/O ports, so that a driver can
//! run in user mode and reach exactly its own device and nothing else.
//! A program starts with the handles its creator gives it. It gets new
//! ones only by receiving a reply port in a message, or by looking a name
//! up, which itself needs the `lookup` permission.

use crate::arch::paging::{AddressSpace, USER_START};
use crate::ipc::Port;
use crate::mem::{frame, PAGE_SIZE};
use crate::sched;
use crate::sync::SpinLock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicI64, AtomicU64, Ordering};

#[cfg(target_arch = "x86_64")]
pub const CODE_BASE: u64 = USER_START;
pub const STACK_TOP: u64 = USER_START + 0x1000_0000;
/// 64 KiB: enough for a program that keeps a screen of text on its stack.
const STACK_PAGES: u64 = 16;
/// Device memory is mapped here, 1 GiB above the code, in a 1 GiB window.
const DEVICE_BASE: u64 = USER_START + 0x4000_0000;
const DEVICE_END: u64 = DEVICE_BASE + 0x4000_0000;

pub const RIGHT_SEND: u8 = 1;
pub const RIGHT_RECV: u8 = 2;

/// What a handle gives access to.
#[derive(Clone)]
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub enum Cap {
    /// A message port, with send and/or receive rights.
    Port { port: Arc<Port>, rights: u8 },
    /// A range of x86 I/O ports, for a driver that runs in user mode
    /// (x86_64 only; ARM devices are reached through memory instead).
    Io { base: u16, len: u16 },
    /// A volume (the boot image or the disk): may list it and open its
    /// files.
    Dir(Volume),
    /// One read-only file: read it, or start it as a program.
    File { name: &'static str, data: FileData },
    /// The console's keyboard: may read what is typed.
    Console,
    /// An interrupt line (0-15), for a driver in user mode to wait on.
    Irq(u8),
    /// May add bytes to what the console reads, as a keyboard driver does.
    Typing,
    /// May move the pointer (mouse) and press its buttons, as a mouse
    /// driver does.
    Pointer,
    /// A range of device memory (a framebuffer, say) that the program may
    /// map into its address space.
    Mem { phys: u64, pages: u64 },
    /// May say which port keyboard bytes go to (the window in focus), as
    /// the display service does.
    KeyFocus,
    /// Memory shared between programs (a window's pixels, say): one
    /// program makes it, passes the handle in a message, and both map it.
    Shared(Arc<Shared>),
}

/// RAM shared between programs. It goes back to the frame allocator when
/// the last handle to it and the last mapping of it are gone.
pub struct Shared {
    pub phys: u64,
    pub pages: u64,
}

/// The largest shared block a program may make (4 MiB).
pub const MAX_SHARED_PAGES: u64 = 1024;

impl Shared {
    /// Makes a block of `pages` zeroed, physically contiguous pages.
    pub fn new(pages: u64) -> Option<Arc<Shared>> {
        if pages == 0 || pages > MAX_SHARED_PAGES {
            return None;
        }
        let phys = crate::mem::frame::alloc_contiguous(pages as usize)?;
        // RAM is identity mapped for the kernel.
        unsafe { core::ptr::write_bytes(phys as *mut u8, 0, (pages * PAGE_SIZE) as usize) };
        Some(Arc::new(Shared { phys, pages }))
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        for i in 0..self.pages {
            crate::mem::frame::free(self.phys + i * PAGE_SIZE);
        }
    }
}

/// The volumes a directory handle can stand for, named as in Neo DOS.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Volume {
    /// `BOOT:`, the boot image in memory.
    Boot,
    /// `DH0:`, the first disk.
    Disk,
}

/// The bytes of an open file: in the boot image, or read from the disk.
#[derive(Clone)]
pub enum FileData {
    Static(&'static [u8]),
    Owned(Arc<[u8]>),
}

impl FileData {
    pub fn bytes(&self) -> &[u8] {
        match self {
            FileData::Static(b) => b,
            FileData::Owned(b) => b,
        }
    }
}

/// What kind of thing a handle stands for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Port,
    Io,
    Dir,
    File,
    Console,
    Irq,
    Typing,
    Pointer,
    Mem,
    Shared,
    KeyFocus,
}

impl Cap {
    pub fn new(port: Arc<Port>, rights: u8) -> Self {
        Cap::Port { port, rights }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn io(base: u16, len: u16) -> Self {
        Cap::Io { base, len }
    }
}

pub struct Process {
    space: Option<AddressSpace>,
    /// Where the program starts.
    entry: u64,
    handles: Vec<Option<Cap>>,
    /// May look ports up by name.
    pub may_lookup: bool,
    /// Where the next device memory mapping goes.
    device_next: u64,
    /// Programs this one started.
    children: Vec<u64>,
    /// Shared memory this program has mapped.
    mapped: Vec<Arc<Shared>>,
}

impl Process {
    /// The port behind `handle`, if the handle exists and has `right`.
    pub fn port(&self, handle: u64, right: u8) -> Option<Arc<Port>> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::Port { port, rights })) if rights & right == right => Some(port.clone()),
            _ => None,
        }
    }

    /// What `handle` is, if it exists.
    pub fn kind(&self, handle: u64) -> Option<Kind> {
        match self.handles.get(handle as usize)? {
            Some(Cap::Port { .. }) => Some(Kind::Port),
            Some(Cap::Io { .. }) => Some(Kind::Io),
            Some(Cap::Dir(_)) => Some(Kind::Dir),
            Some(Cap::File { .. }) => Some(Kind::File),
            Some(Cap::Console) => Some(Kind::Console),
            Some(Cap::Irq(_)) => Some(Kind::Irq),
            Some(Cap::Typing) => Some(Kind::Typing),
            Some(Cap::Pointer) => Some(Kind::Pointer),
            Some(Cap::Mem { .. }) => Some(Kind::Mem),
            Some(Cap::Shared(_)) => Some(Kind::Shared),
            Some(Cap::KeyFocus) => Some(Kind::KeyFocus),
            None => None,
        }
    }

    /// The file behind `handle`, if it is a file handle.
    pub fn file(&self, handle: u64) -> Option<(&'static str, FileData)> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::File { name, data })) => Some((name, data.clone())),
            _ => None,
        }
    }

    /// The volume behind `handle`, if it is a directory handle.
    pub fn volume(&self, handle: u64) -> Option<Volume> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::Dir(v))) => Some(*v),
            _ => None,
        }
    }

    /// The interrupt line behind `handle`, if it is an interrupt handle.
    pub fn irq_line(&self, handle: u64) -> Option<usize> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::Irq(line))) => Some(*line as usize),
            _ => None,
        }
    }

    /// Maps the device memory behind `handle` into this program; returns
    /// where. Each call maps it again at a new address.
    pub fn map_device(&mut self, handle: u64) -> Option<u64> {
        let (phys, pages, shared) = match self.handles.get(handle as usize).cloned() {
            Some(Some(Cap::Mem { phys, pages })) => (phys, pages, None),
            Some(Some(Cap::Shared(sh))) => (sh.phys, sh.pages, Some(sh)),
            _ => return None,
        };
        let at = self.device_next;
        if at + pages * PAGE_SIZE > DEVICE_END {
            return None;
        }
        let space = self.space.as_mut()?;
        for i in 0..pages {
            let (v, f) = (at + i * PAGE_SIZE, phys + i * PAGE_SIZE);
            if shared.is_some() {
                space.map_user_shared_4k(v, f);
            } else {
                space.map_user_device_4k(v, f);
            }
        }
        // A mapping keeps shared memory alive even if the handle is closed.
        if let Some(sh) = shared {
            self.mapped.push(sh);
        }
        self.device_next = at + (pages + 1) * PAGE_SIZE; // one page apart
        Some(at)
    }

    /// The size of the shared memory behind `handle`, in bytes.
    pub fn shared_bytes(&self, handle: u64) -> Option<u64> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::Shared(sh))) => Some(sh.pages * PAGE_SIZE),
            _ => None,
        }
    }

    /// A copy of the capability behind `handle`, to hand to a new program.
    pub fn cap(&self, handle: u64) -> Option<Cap> {
        self.handles.get(handle as usize)?.clone()
    }

    /// Remembers a program this one started, so that it may wait for it.
    pub fn add_child(&mut self, id: u64) {
        self.children.push(id);
    }

    pub fn is_child(&self, id: u64) -> bool {
        self.children.contains(&id)
    }

    /// The I/O port at `offset` inside the range behind `handle`.
    #[cfg(target_arch = "x86_64")]
    pub fn io_port(&self, handle: u64, offset: u64) -> Option<u16> {
        match self.handles.get(handle as usize) {
            Some(Some(Cap::Io { base, len })) if offset < *len as u64 => Some(base + offset as u16),
            _ => None,
        }
    }

    /// Adds a capability; returns its handle (reusing a free slot).
    pub fn grant(&mut self, cap: Cap) -> u64 {
        if let Some(i) = self.handles.iter().position(|h| h.is_none()) {
            self.handles[i] = Some(cap);
            i as u64
        } else {
            self.handles.push(Some(cap));
            (self.handles.len() - 1) as u64
        }
    }

    pub fn close(&mut self, handle: u64) -> bool {
        match self.handles.get_mut(handle as usize) {
            Some(slot @ Some(_)) => {
                *slot = None;
                true
            }
            _ => false,
        }
    }

    pub fn root(&self) -> u64 {
        self.space.as_ref().unwrap().root()
    }

    pub fn space(&self) -> &AddressSpace {
        self.space.as_ref().unwrap()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Some(space) = self.space.take() {
            space.destroy_user();
        }
    }
}

/// Builds an address space holding a copy of `image` (read-only, at
/// CODE_BASE) and an empty stack below STACK_TOP.
#[cfg(target_arch = "x86_64")]
fn load(image: &[u8]) -> Process {
    let mut space = AddressSpace::new_user();
    let pages = (image.len() as u64).div_ceil(PAGE_SIZE).max(1);
    for i in 0..pages {
        let f = frame::alloc().expect("out of memory loading a program");
        let dst = f as *mut u8; // physical memory is identity mapped
        unsafe { dst.write_bytes(0, PAGE_SIZE as usize) };
        let from = (i * PAGE_SIZE) as usize;
        let n = (image.len() - from).min(PAGE_SIZE as usize);
        unsafe { core::ptr::copy_nonoverlapping(image.as_ptr().add(from), dst, n) };
        space.map_user_4k(CODE_BASE + i * PAGE_SIZE, f, false);
    }
    map_stack(&mut space);
    Process {
        space: Some(space),
        entry: CODE_BASE,
        handles: Vec::new(),
        may_lookup: false,
        mapped: Vec::new(),
        device_next: DEVICE_BASE,
        children: Vec::new(),
    }
}

fn map_stack(space: &mut AddressSpace) {
    for i in 1..=STACK_PAGES {
        let f = frame::alloc().expect("out of memory for a program stack");
        unsafe { (f as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
        space.map_user_4k(STACK_TOP - i * PAGE_SIZE, f, true);
    }
}

/// Builds an address space from an ELF executable. Each segment must lie in
/// the user part below the stack, start on its own page and not overlap
/// another; the entry point must be in an executable segment.
fn load_elf(file: &[u8]) -> Result<Process, &'static str> {
    use crate::arch::paging::is_user;
    use crate::elf::{self, PF_W, PF_X};

    let img = elf::parse(file)?;
    let below_stack = STACK_TOP - STACK_PAGES * PAGE_SIZE;
    let mut spans: Vec<(u64, u64)> = Vec::new();
    for s in &img.segments {
        let end = s.vaddr + s.mem_size;
        if !is_user(s.vaddr) || end > below_stack {
            return Err("segment outside the program's part of memory");
        }
        if s.vaddr % PAGE_SIZE != 0 {
            return Err("segment does not start on a page");
        }
        let span = (s.vaddr, end.next_multiple_of(PAGE_SIZE));
        if spans.iter().any(|&(a, b)| span.0 < b && a < span.1) {
            return Err("segments overlap");
        }
        spans.push(span);
    }
    let entry_ok =
        img.segments.iter().any(|s| s.flags & PF_X != 0 && (s.vaddr..s.vaddr + s.mem_size).contains(&img.entry));
    if !entry_ok {
        return Err("entry point is not in executable code");
    }

    let mut space = AddressSpace::new_user();
    for s in &img.segments {
        let pages = s.mem_size.div_ceil(PAGE_SIZE);
        for i in 0..pages {
            let Some(f) = frame::alloc() else {
                space.destroy_user();
                return Err("out of memory");
            };
            let dst = f as *mut u8; // physical memory is identity mapped
            unsafe { dst.write_bytes(0, PAGE_SIZE as usize) };
            let page_off = i * PAGE_SIZE;
            if page_off < s.file_size {
                let n = (s.file_size - page_off).min(PAGE_SIZE) as usize;
                let src = &file[(s.offset + page_off) as usize..][..n];
                unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst, n) };
            }
            space.map_user_4k(s.vaddr + page_off, f, s.flags & PF_W != 0);
        }
    }
    map_stack(&mut space);
    Ok(Process {
        space: Some(space),
        entry: img.entry,
        handles: Vec::new(),
        may_lookup: false,
        mapped: Vec::new(),
        device_next: DEVICE_BASE,
        children: Vec::new(),
    })
}

/// Starts a program from a raw machine-code image with the given
/// capabilities as handles 0, 1, 2... Returns its thread id.
#[cfg(target_arch = "x86_64")]
pub fn spawn(name: &'static str, image: &[u8], caps: Vec<Cap>, may_lookup: bool) -> u64 {
    start(name, load(image), caps, may_lookup)
}

/// Starts a program from an ELF executable. Returns its thread id, or why
/// the file was refused.
pub fn spawn_elf(name: &'static str, file: &[u8], caps: Vec<Cap>, may_lookup: bool) -> Result<u64, &'static str> {
    Ok(start(name, load_elf(file)?, caps, may_lookup))
}

fn start(name: &'static str, mut p: Process, caps: Vec<Cap>, may_lookup: bool) -> u64 {
    for c in caps {
        p.grant(c);
    }
    p.may_lookup = may_lookup;
    sched::spawn_process(name, p, user_entry)
}

/// First code of a program thread: drops to user mode.
fn user_entry() {
    let entry = sched::with_current_process(|p| p.entry).expect("program thread without a program");
    crate::arch::enter_user(entry, STACK_TOP)
}

/// How programs ended, for diagnostics and the self-test.
pub static STOPPED: AtomicU64 = AtomicU64::new(0);
pub static LAST_EXIT: AtomicI64 = AtomicI64::new(i64::MIN);

/// The exit code reported for a program the kernel had to stop.
pub const STOPPED_CODE: i64 = -255;

/// How recent programs ended: (thread id, exit code), the last 64.
static EXITS: SpinLock<VecDeque<(u64, i64)>> = SpinLock::new(VecDeque::new());

fn record_exit(code: i64) {
    let mut exits = EXITS.lock();
    if exits.len() == 64 {
        exits.pop_front();
    }
    exits.push_back((sched::current_id(), code));
}

/// Waits until program `id` has ended; returns its exit code.
pub fn wait(id: u64) -> i64 {
    loop {
        if let Some(&(_, code)) = EXITS.lock().iter().find(|e| e.0 == id) {
            return code;
        }
        sched::sleep(1);
    }
}

/// A program asked to end.
pub fn exit(code: i64) -> ! {
    LAST_EXIT.store(code, Ordering::Relaxed);
    record_exit(code);
    sched::exit()
}

/// A program did something it may not: report it and end that program only.
/// `pc` is where it happened, `detail` the faulting address or error code.
pub fn stop(reason: &str, pc: u64, detail: u64) -> ! {
    crate::kprintln!("Program {} stopped: {} at {:#x} ({:#x})", sched::current_name(), reason, pc, detail);
    STOPPED.fetch_add(1, Ordering::Relaxed);
    record_exit(STOPPED_CODE);
    sched::exit()
}
