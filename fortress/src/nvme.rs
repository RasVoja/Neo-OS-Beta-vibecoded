//! A disk on an NVMe controller (PCI class 01/08/02), the kind of disk a
//! modern PC has.
//!
//! Like the virtio disk it is a bootstrap driver in the kernel, and it
//! works in the simplest way: one admin queue and one I/O queue, one
//! command at a time, completions found by polling (no interrupts), and
//! every transfer through one page of memory (at most eight 512-byte
//! sectors, so no PRP lists). It takes the first namespace, if its sectors
//! are 512 bytes.
//!
//! A PC's NVMe disk usually holds someone's own system, so the driver
//! reads it but never writes to it unless sector 0 holds a ustar header,
//! that is, unless it is a disk made for Neo (diskfs.rs keeps files in a
//! ustar archive).

use crate::mem::{frame, PAGE_SIZE};
use crate::pci;
use crate::sync::SpinLock;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{fence, Ordering};

pub const SECTOR: usize = 512;
/// Most sectors one command moves: one page.
const MAX_SECTORS: usize = 8;
/// Entries in each queue (a page holds 64 submission entries).
const DEPTH: usize = 16;

// Controller registers.
const CAP: usize = 0x00;
const CC: usize = 0x14;
const CSTS: usize = 0x1C;
const AQA: usize = 0x24;
const ASQ: usize = 0x28;
const ACQ: usize = 0x30;
const DOORBELLS: usize = 0x1000;
const CC_ENABLE: u32 = 1;
/// 64-byte submission entries, 16-byte completion entries, 4 KiB pages.
const CC_SETTINGS: u32 = (6 << 16) | (4 << 20);
const CSTS_READY: u32 = 1;

// Commands.
const ADMIN_CREATE_SQ: u32 = 0x01;
const ADMIN_CREATE_CQ: u32 = 0x05;
const ADMIN_IDENTIFY: u32 = 0x06;
const IO_WRITE: u32 = 0x01;
const IO_READ: u32 = 0x02;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Command {
    /// Opcode in the low byte, the command's number in the high half.
    dw0: u32,
    nsid: u32,
    _reserved: [u32; 2],
    metadata: u64,
    prp1: u64,
    prp2: u64,
    dw10: u32,
    dw11: u32,
    dw12: u32,
    dw13: u32,
    dw14: u32,
    dw15: u32,
}

/// A queue pair: where the entries are and how far we have got.
struct Queue {
    sq: u64,
    cq: u64,
    sq_tail: usize,
    cq_head: usize,
    /// The phase the next completion has when it is new.
    phase: u32,
    /// The doorbells' offsets in the register block.
    sq_bell: usize,
    cq_bell: usize,
    depth: usize,
}

pub struct Nvme {
    regs: usize,
    admin: Queue,
    io: Queue,
    /// A page the data moves through.
    bounce: u64,
    pub sectors: u64,
    pub read_only: bool,
    /// Was an empty disk, taken for DH0:.
    pub blank: bool,
    next_id: u32,
}

/// Registers for the controller at `regs`, before the queues exist.
fn wait_ready(regs: usize, ready: bool, units_of_500ms: u64) -> Result<(), &'static str> {
    let csts = |regs: usize| unsafe { read_volatile((regs + CSTS) as *const u32) };
    // About 500 ms per unit: the timer is not running everywhere this runs,
    // so count reads (each takes about a microsecond or more).
    for _ in 0..units_of_500ms.max(1) * 500_000 {
        if (csts(regs) & CSTS_READY != 0) == ready {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err("the controller did not answer")
}

fn zeroed_page() -> Result<u64, &'static str> {
    let p = frame::alloc().ok_or("no memory for the queues")?;
    unsafe { (p as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
    Ok(p)
}

impl Queue {
    fn new(id: usize, stride: usize, depth: usize) -> Result<Queue, &'static str> {
        Ok(Queue {
            sq: zeroed_page()?,
            cq: zeroed_page()?,
            sq_tail: 0,
            cq_head: 0,
            phase: 1,
            sq_bell: DOORBELLS + (2 * id) * stride,
            cq_bell: DOORBELLS + (2 * id + 1) * stride,
            depth,
        })
    }
}

impl Nvme {
    /// Sends one command on `io` (or the admin queue) and waits for its
    /// completion; returns the completion's status code (0 = done).
    fn run(&mut self, admin: bool, mut cmd: Command) -> Result<(), &'static str> {
        self.next_id = self.next_id.wrapping_add(1);
        cmd.dw0 = cmd.dw0 & 0xFF | (self.next_id & 0xFFFF) << 16;
        let regs = self.regs;
        let q = if admin { &mut self.admin } else { &mut self.io };
        unsafe { write_volatile((q.sq as *mut Command).add(q.sq_tail), cmd) };
        q.sq_tail = (q.sq_tail + 1) % q.depth;
        fence(Ordering::SeqCst);
        unsafe { write_volatile((regs + q.sq_bell) as *mut u32, q.sq_tail as u32) };
        // Poll for the completion entry: its phase bit flips when it is new.
        let entry = unsafe { (q.cq as *const [u32; 4]).add(q.cq_head) };
        let mut spins = 0u64;
        let dw3 = loop {
            let dw3 = unsafe { read_volatile(&(*entry)[3]) };
            if dw3 >> 16 & 1 == q.phase {
                break dw3;
            }
            spins += 1;
            if spins > 200_000_000 {
                return Err("the disk did not answer");
            }
            core::hint::spin_loop();
        };
        q.cq_head += 1;
        if q.cq_head == q.depth {
            q.cq_head = 0;
            q.phase ^= 1;
        }
        fence(Ordering::SeqCst);
        unsafe { write_volatile((regs + q.cq_bell) as *mut u32, q.cq_head as u32) };
        // Status: bits 31:17 (code in the low 8 of them, then the type).
        if dw3 >> 17 != 0 {
            return Err("the disk reported an error");
        }
        Ok(())
    }

    fn init(f: pci::Function) -> Result<Nvme, &'static str> {
        // Memory space and bus mastering (the controller reads and writes
        // our memory).
        f.enable(0x6);
        let bar = f.mem_bar(0).ok_or("no register block")?;
        let regs = pci::map(bar)?;
        let cap = unsafe { read_volatile((regs + CAP) as *const u64) };
        let timeout = (cap >> 24) & 0xFF;
        let stride = 4usize << ((cap >> 32) & 0xF);
        if cap >> 37 & 1 == 0 {
            return Err("the controller does not speak the NVM command set");
        }
        if 4096 < 1usize << (12 + ((cap >> 48) & 0xF)) {
            return Err("the controller needs larger pages");
        }

        // Stop it, give it the admin queue, start it.
        unsafe {
            write_volatile((regs + CC) as *mut u32, 0);
        }
        wait_ready(regs, false, timeout)?;
        let admin = Queue::new(0, stride, DEPTH)?;
        unsafe {
            write_volatile((regs + AQA) as *mut u32, ((DEPTH as u32 - 1) << 16) | (DEPTH as u32 - 1));
            write_volatile((regs + ASQ) as *mut u64, admin.sq);
            write_volatile((regs + ACQ) as *mut u64, admin.cq);
            write_volatile((regs + CC) as *mut u32, CC_SETTINGS | CC_ENABLE);
        }
        wait_ready(regs, true, timeout)?;

        let io = Queue::new(1, stride, DEPTH)?;
        let bounce = zeroed_page()?;
        let mut d = Nvme { regs, admin, io, bounce, sectors: 0, read_only: true, blank: false, next_id: 0 };

        // The I/O queues: completion first, then the submission queue that
        // reports to it. Both are one contiguous piece of memory.
        let cq = Command {
            dw0: ADMIN_CREATE_CQ,
            prp1: d.io.cq,
            dw10: ((DEPTH as u32 - 1) << 16) | 1,
            dw11: 1,
            ..Default::default()
        };
        d.run(true, cq)?;
        let sq = Command {
            dw0: ADMIN_CREATE_SQ,
            prp1: d.io.sq,
            dw10: ((DEPTH as u32 - 1) << 16) | 1,
            dw11: (1 << 16) | 1,
            ..Default::default()
        };
        d.run(true, sq)?;

        // Namespace 1: its size and sector size.
        unsafe { (d.bounce as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
        let identify = Command { dw0: ADMIN_IDENTIFY, nsid: 1, prp1: d.bounce, dw10: 0, ..Default::default() };
        d.run(true, identify)?;
        let ns = d.bounce as *const u8;
        let size = unsafe { read_volatile(ns as *const u64) };
        let format = unsafe { read_volatile(ns.add(26)) } & 0xF;
        let lbaf = unsafe { read_volatile(ns.add(128 + 4 * format as usize) as *const u32) };
        let sector_bits = (lbaf >> 16) & 0xFF;
        if sector_bits != 9 {
            return Err("only disks with 512-byte sectors are supported");
        }
        if size == 0 {
            return Err("the disk is empty");
        }
        d.sectors = size;
        // Writable only if it is a disk made for Neo: a ustar header in
        // sector 0.
        let mut first = [0u8; SECTOR];
        d.transfer(0, Some(&mut first), None)?;
        d.read_only = &first[257..262] != b"ustar";
        // A blank disk (the first MiB all zeros, so no partition table, no
        // file system, nothing of anyone's) is an empty Neo archive: it is
        // used as DH0: and written. Anything else stays read-only.
        if d.read_only && size >= 131_072 && first.iter().all(|&b| b == 0) {
            let mut chunk = [0u8; 8 * SECTOR];
            let mut blank = true;
            for at in (8..2048u64).step_by(8) {
                d.transfer(at, Some(&mut chunk), None)?;
                if chunk.iter().any(|&b| b != 0) {
                    blank = false;
                    break;
                }
            }
            if blank {
                d.read_only = false;
                d.blank = true;
            }
        }
        Ok(d)
    }

    /// One command's worth: up to eight sectors.
    fn piece(&mut self, sector: u64, count: usize, write: bool) -> Result<(), &'static str> {
        let cmd = Command {
            dw0: if write { IO_WRITE } else { IO_READ },
            nsid: 1,
            prp1: self.bounce,
            dw10: sector as u32,
            dw11: (sector >> 32) as u32,
            dw12: count as u32 - 1,
            ..Default::default()
        };
        self.run(false, cmd)
    }

    fn transfer(
        &mut self,
        sector: u64,
        mut output: Option<&mut [u8]>,
        input: Option<&[u8]>,
    ) -> Result<(), &'static str> {
        let len = output.as_ref().map(|b| b.len()).or(input.map(|b| b.len())).unwrap_or(0);
        if len == 0 || len % SECTOR != 0 {
            return Err("a transfer is a whole number of sectors");
        }
        if sector + (len / SECTOR) as u64 > self.sectors && self.sectors != 0 {
            return Err("beyond the end of the disk");
        }
        let mut done = 0;
        while done < len {
            let n = (len - done).min(MAX_SECTORS * SECTOR);
            let at = sector + (done / SECTOR) as u64;
            if let Some(data) = input {
                // Safe: the bounce page is ours, a page long.
                unsafe { core::ptr::copy_nonoverlapping(data[done..].as_ptr(), self.bounce as *mut u8, n) };
                self.piece(at, n / SECTOR, true)?;
            } else {
                self.piece(at, n / SECTOR, false)?;
                if let Some(buf) = output.as_deref_mut() {
                    unsafe { core::ptr::copy_nonoverlapping(self.bounce as *const u8, buf[done..].as_mut_ptr(), n) };
                }
            }
            done += n;
        }
        Ok(())
    }

    pub fn read(&mut self, sector: u64, buf: &mut [u8]) -> Result<(), &'static str> {
        self.transfer(sector, Some(buf), None)
    }

    pub fn write(&mut self, sector: u64, buf: &[u8]) -> Result<(), &'static str> {
        if self.read_only {
            return Err("the disk is read-only");
        }
        self.transfer(sector, None, Some(buf))
    }
}

static DISK: SpinLock<Option<Nvme>> = SpinLock::new(None);

/// Looks for NVMe disks and sets them up; returns the size in sectors of
/// the one used: a Neo disk first, then a blank one, otherwise the first
/// disk, read-only.
pub fn init() -> Option<Result<u64, &'static str>> {
    let funcs = pci::find_all(|f| f.class >> 8 == 0x0108 && f.class & 0xFF == 0x02);
    if funcs.is_empty() {
        return None;
    }
    let mut best: Option<(u8, Nvme)> = None;
    let mut error = "no usable NVMe disk";
    for f in funcs {
        match Nvme::init(f) {
            Ok(d) => {
                let rank = if d.read_only {
                    0
                } else if d.blank {
                    1
                } else {
                    2
                };
                if best.as_ref().is_none_or(|(r, _)| rank > *r) {
                    best = Some((rank, d));
                }
            }
            Err(e) => error = e,
        }
    }
    Some(match best {
        Some((_, d)) => {
            let sectors = d.sectors;
            *DISK.lock() = Some(d);
            Ok(sectors)
        }
        None => Err(error),
    })
}

pub fn blank() -> bool {
    DISK.lock().as_ref().is_some_and(|d| d.blank)
}

pub fn read(sector: u64, buf: &mut [u8]) -> Option<Result<(), &'static str>> {
    DISK.lock().as_mut().map(|d| d.read(sector, buf))
}

pub fn write(sector: u64, buf: &[u8]) -> Option<Result<(), &'static str>> {
    DISK.lock().as_mut().map(|d| d.write(sector, buf))
}

pub fn writable() -> bool {
    DISK.lock().as_ref().is_some_and(|d| !d.read_only)
}

pub fn sectors() -> Option<u64> {
    DISK.lock().as_ref().map(|d| d.sectors)
}
