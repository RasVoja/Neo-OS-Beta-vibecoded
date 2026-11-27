//! A virtio block device (a disk), legacy interface, polled.
//!
//! QEMU offers the same device through PCI on x86_64 (`virtio-blk-pci`,
//! with `disable-modern=on` for the legacy interface on I/O ports) and
//! through memory-mapped registers on AArch64 (`virtio-blk-device` on the
//! `virt` machine's virtio-mmio slots). Only the register access differs;
//! the queue of requests is the same.
//!
//! One queue, one request at a time: a 3-part chain (request header, data,
//! status byte) is put in the queue, the device is told, and the driver
//! waits for the device to report it done. Interrupts come later.
//!
//! This driver lives in the kernel for now, as a bootstrap; on the way to
//! the real Fortress it moves to a driver process in user mode.

use crate::mem::{frame, PAGE_SIZE};
use crate::sync::SpinLock;
use core::sync::atomic::{fence, Ordering};

pub const SECTOR: usize = 512;
/// Most sectors read by one request.
const MAX_SECTORS: usize = 8;

const STATUS_ACK: u8 = 1;
const STATUS_DRIVER: u8 = 2;
const STATUS_DRIVER_OK: u8 = 4;
const STATUS_FAILED: u8 = 128;

const DESC_NEXT: u16 = 1;
const DESC_WRITE: u16 = 2;
const REQ_READ: u32 = 0;

/// Where the device's registers are.
#[derive(Clone, Copy)]
enum Transport {
    #[cfg(target_arch = "x86_64")]
    Pci { io: u16 },
    #[cfg(target_arch = "aarch64")]
    Mmio { base: usize },
}

// Legacy register offsets: (PCI I/O, MMIO).
const HOST_FEATURES: (u16, usize) = (0x00, 0x010);
const GUEST_FEATURES: (u16, usize) = (0x04, 0x020);
const QUEUE_PFN: (u16, usize) = (0x08, 0x040);
const QUEUE_NUM: (u16, usize) = (0x0C, 0x038);
const QUEUE_SEL: (u16, usize) = (0x0E, 0x030);
const QUEUE_NOTIFY: (u16, usize) = (0x10, 0x050);
const STATUS: (u16, usize) = (0x12, 0x070);
const CONFIG: (u16, usize) = (0x14, 0x100);
#[cfg(target_arch = "aarch64")]
const MMIO_QUEUE_NUM_MAX: usize = 0x034;
#[cfg(target_arch = "aarch64")]
const MMIO_QUEUE_ALIGN: usize = 0x03C;
#[cfg(target_arch = "aarch64")]
const MMIO_GUEST_PAGE_SIZE: usize = 0x028;

impl Transport {
    fn read32(self, reg: (u16, usize)) -> u32 {
        match self {
            #[cfg(target_arch = "x86_64")]
            Transport::Pci { io } => unsafe { crate::arch::inl(io + reg.0) },
            #[cfg(target_arch = "aarch64")]
            Transport::Mmio { base } => unsafe { ((base + reg.1) as *const u32).read_volatile() },
        }
    }

    fn write32(self, reg: (u16, usize), v: u32) {
        match self {
            #[cfg(target_arch = "x86_64")]
            Transport::Pci { io } => unsafe { crate::arch::outl(io + reg.0, v) },
            #[cfg(target_arch = "aarch64")]
            Transport::Mmio { base } => unsafe { ((base + reg.1) as *mut u32).write_volatile(v) },
        }
    }

    /// The 16-bit registers (queue size, select, notify) and the status byte
    /// are narrower on PCI; on MMIO every register is 32 bits wide.
    fn write16(self, reg: (u16, usize), v: u16) {
        match self {
            #[cfg(target_arch = "x86_64")]
            Transport::Pci { io } => unsafe { crate::arch::outw(io + reg.0, v) },
            #[cfg(target_arch = "aarch64")]
            Transport::Mmio { .. } => self.write32(reg, v as u32),
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn read16(self, reg: (u16, usize)) -> u16 {
        match self {
            #[cfg(target_arch = "x86_64")]
            Transport::Pci { io } => unsafe { crate::arch::inw(io + reg.0) },
        }
    }

    fn set_status(self, v: u8) {
        match self {
            #[cfg(target_arch = "x86_64")]
            Transport::Pci { io } => unsafe { crate::arch::outb(io + STATUS.0, v) },
            #[cfg(target_arch = "aarch64")]
            Transport::Mmio { .. } => self.write32(STATUS, v as u32),
        }
    }

    fn config_u64(self, offset: u16) -> u64 {
        let reg = |o: u16| (CONFIG.0 + o, CONFIG.1 + o as usize);
        self.read32(reg(offset)) as u64 | (self.read32(reg(offset + 4)) as u64) << 32
    }
}

/// Finds a virtio block device: PCI vendor 0x1AF4, legacy device 0x1001.
#[cfg(target_arch = "x86_64")]
fn find() -> Option<Transport> {
    // The legacy interface is on only when BAR 0 is an I/O BAR.
    let f = crate::pci::find(|f| f.id == 0x1001_1AF4 && f.io_bar(0).is_some())?;
    // Turn on I/O decoding and bus mastering (the device reads and writes
    // memory itself).
    f.enable(0x5);
    Some(Transport::Pci { io: f.io_bar(0)? })
}

/// Finds a virtio block device in the `virt` machine's 32 virtio-mmio slots.
#[cfg(target_arch = "aarch64")]
fn find() -> Option<Transport> {
    for slot in 0..32 {
        let base = 0x0A00_0000 + slot * 0x200;
        let reg = |o: usize| unsafe { ((base + o) as *const u32).read_volatile() };
        // Magic "virt", legacy version 1, device type 2 (block).
        if reg(0) == 0x7472_6976 && reg(4) == 1 && reg(8) == 2 {
            return Some(Transport::Mmio { base });
        }
    }
    None
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Desc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct Request {
    kind: u32,
    reserved: u32,
    sector: u64,
}

pub struct Disk {
    t: Transport,
    size: u16,
    /// The queue (descriptors, available ring, used ring), contiguous.
    ring: u64,
    /// One page for the request header, the status byte and the data.
    io: u64,
    last_used: u16,
    pub sectors: u64,
}

// The queue memory belongs to the driver; access goes through the lock.
unsafe impl Send for Disk {}

/// Bytes of a legacy queue of `n` entries: descriptors and available ring,
/// then the used ring on the next page.
fn ring_bytes(n: usize) -> usize {
    let first = (16 * n + 6 + 2 * n).next_multiple_of(PAGE_SIZE as usize);
    first + (6 + 8 * n).next_multiple_of(PAGE_SIZE as usize)
}

impl Disk {
    fn init(t: Transport) -> Result<Disk, &'static str> {
        t.set_status(0); // reset
        t.set_status(STATUS_ACK);
        t.set_status(STATUS_ACK | STATUS_DRIVER);
        let _ = t.read32(HOST_FEATURES);
        t.write32(GUEST_FEATURES, 0); // nothing optional needed

        t.write16(QUEUE_SEL, 0);
        #[cfg(target_arch = "aarch64")]
        let size = {
            let Transport::Mmio { base } = t;
            let max = unsafe { ((base + MMIO_QUEUE_NUM_MAX) as *const u32).read_volatile() };
            let size = max.min(64);
            t.write32(QUEUE_NUM, size);
            unsafe {
                ((base + MMIO_GUEST_PAGE_SIZE) as *mut u32).write_volatile(PAGE_SIZE as u32);
                ((base + MMIO_QUEUE_ALIGN) as *mut u32).write_volatile(PAGE_SIZE as u32);
            }
            size as u16
        };
        #[cfg(target_arch = "x86_64")]
        let size = t.read16(QUEUE_NUM); // fixed by the device on legacy PCI
        if size == 0 {
            t.set_status(STATUS_FAILED);
            return Err("the device has no queue 0");
        }

        let pages = ring_bytes(size as usize) / PAGE_SIZE as usize;
        let ring = frame::alloc_contiguous(pages).ok_or("no memory for the queue")?;
        unsafe { (ring as *mut u8).write_bytes(0, pages * PAGE_SIZE as usize) };
        let io =
            frame::alloc_contiguous(MAX_SECTORS * SECTOR / PAGE_SIZE as usize + 1).ok_or("no memory for requests")?;
        t.write32(QUEUE_PFN, (ring / PAGE_SIZE) as u32);
        t.set_status(STATUS_ACK | STATUS_DRIVER | STATUS_DRIVER_OK);

        let sectors = t.config_u64(0);
        Ok(Disk { t, size, ring, io, last_used: 0, sectors })
    }

    fn desc(&self) -> *mut Desc {
        self.ring as *mut Desc
    }

    fn avail(&self) -> *mut u16 {
        (self.ring + 16 * self.size as u64) as *mut u16
    }

    fn used_idx(&self) -> *const u16 {
        let used = self.ring + (16 * self.size as u64 + 6 + 2 * self.size as u64).next_multiple_of(PAGE_SIZE);
        (used + 2) as *const u16
    }

    /// Reads `buf.len() / 512` sectors (at most 8) starting at `sector`.
    pub fn read(&mut self, sector: u64, buf: &mut [u8]) -> Result<(), &'static str> {
        let count = buf.len() / SECTOR;
        if count == 0 || count > MAX_SECTORS || buf.len() % SECTOR != 0 {
            return Err("bad read size");
        }
        if sector + count as u64 > self.sectors {
            return Err("read past the end of the disk");
        }
        // Layout of the I/O page(s): header at 0, status at 16, data at 512.
        let header = self.io;
        let status = self.io + 16;
        let data = self.io + SECTOR as u64;
        unsafe {
            (header as *mut Request).write(Request { kind: REQ_READ, reserved: 0, sector });
            (status as *mut u8).write_volatile(0xFF);
            let d = self.desc();
            d.add(0).write(Desc { addr: header, len: 16, flags: DESC_NEXT, next: 1 });
            d.add(1).write(Desc { addr: data, len: (count * SECTOR) as u32, flags: DESC_NEXT | DESC_WRITE, next: 2 });
            d.add(2).write(Desc { addr: status, len: 1, flags: DESC_WRITE, next: 0 });

            // Offer descriptor 0 in the available ring, then publish it.
            let avail = self.avail();
            let idx = avail.add(1).read_volatile();
            avail.add(2 + (idx % self.size) as usize).write_volatile(0);
            fence(Ordering::SeqCst);
            avail.add(1).write_volatile(idx.wrapping_add(1));
            fence(Ordering::SeqCst);
        }
        self.t.write16(QUEUE_NOTIFY, 0);

        // Wait for the device (a request takes microseconds under QEMU).
        let mut spins: u64 = 0;
        while unsafe { self.used_idx().read_volatile() } == self.last_used {
            spins += 1;
            if spins > 100_000_000 {
                return Err("the disk did not answer");
            }
            core::hint::spin_loop();
        }
        fence(Ordering::SeqCst);
        self.last_used = self.last_used.wrapping_add(1);
        if unsafe { (status as *const u8).read_volatile() } != 0 {
            return Err("the disk reported an error");
        }
        unsafe { core::ptr::copy_nonoverlapping(data as *const u8, buf.as_mut_ptr(), buf.len()) };
        Ok(())
    }
}

static DISK: SpinLock<Option<Disk>> = SpinLock::new(None);

/// Looks for a disk and sets it up; returns its size in sectors.
pub fn init() -> Option<Result<u64, &'static str>> {
    let t = find()?;
    Some(Disk::init(t).map(|d| {
        let sectors = d.sectors;
        *DISK.lock() = Some(d);
        sectors
    }))
}

/// Reads sectors from the disk, if there is one.
pub fn read(sector: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    DISK.lock().as_mut().ok_or("no disk")?.read(sector, buf)
}

pub fn sectors() -> Option<u64> {
    DISK.lock().as_ref().map(|d| d.sectors)
}
