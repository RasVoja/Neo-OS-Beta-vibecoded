//! USB through an xHCI controller (USB 3 host controller interface), for
//! the keyboard and mouse of a real PC.
//!
//! This first part brings the controller up: it finds it on the PCI bus
//! (class 0C/03/30), resets it, gives it the device context table (with
//! scratchpad pages when the controller asks for them), a command ring and
//! an event ring, starts it, and proves the rings work with a No-Op
//! command. Then it reads the root ports and hands each one with something
//! plugged in to usb.rs, which sets the device up.
//!
//! Like the virtio drivers it lives in the kernel as a bootstrap, and it
//! polls instead of taking interrupts (MSI comes later). It moves to user
//! mode, or to a Genode driver, with the rest (decision 0001).

use crate::mem::{frame, PAGE_SIZE};
use crate::sync::SpinLock;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{fence, Ordering};

// Capability registers.
const CAPLENGTH: usize = 0x00;
const HCSPARAMS1: usize = 0x04;
const HCSPARAMS2: usize = 0x08;
const HCCPARAMS1: usize = 0x10;
const DBOFF: usize = 0x14;
const RTSOFF: usize = 0x18;
// Operational registers.
const USBCMD: usize = 0x00;
const USBSTS: usize = 0x04;
const CRCR: usize = 0x18;
const DCBAAP: usize = 0x30;
const CONFIG: usize = 0x38;
const PORTSC: usize = 0x400;
const CMD_RUN: u32 = 1 << 0;
const CMD_RESET: u32 = 1 << 1;
const STS_HALTED: u32 = 1 << 0;
const STS_NOT_READY: u32 = 1 << 11;
// Interrupter 0, in the runtime registers.
const IR0: usize = 0x20;
const ERSTSZ: usize = 0x08;
const ERSTBA: usize = 0x10;
const ERDP: usize = 0x18;
const ERDP_BUSY: u64 = 1 << 3;
// Port status and control.
pub const PORT_CONNECTED: u32 = 1 << 0;
pub const PORT_ENABLED: u32 = 1 << 1;
pub const PORT_RESET: u32 = 1 << 4;
pub const PORT_RESET_CHANGE: u32 = 1 << 21;
/// The bits a write must carry over: power, indicator and wake settings.
pub const PORT_KEEP: u32 = 1 << 9 | 0b11 << 14 | 0b111 << 25;

// TRB types.
const TRB_LINK: u32 = 6;
const TRB_NOOP_CMD: u32 = 23;
pub const TRB_COMMAND_DONE: u32 = 33;
const LINK_TOGGLE: u32 = 1 << 1;
pub const COMPLETION_SUCCESS: u32 = 1;

/// TRBs in each ring (one page of 16-byte TRBs).
const RING: usize = (PAGE_SIZE / 16) as usize;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Trb {
    pub param: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn kind(&self) -> u32 {
        (self.control >> 10) & 0x3F
    }
    pub fn code(&self) -> u32 {
        self.status >> 24
    }
}

/// A ring the driver writes and the controller reads (commands, transfers).
pub struct ProducerRing {
    pub base: u64,
    index: usize,
    cycle: u32,
}

impl ProducerRing {
    pub fn new() -> Option<Self> {
        let base = frame::alloc()?;
        unsafe { (base as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
        // The last TRB links back to the start and flips the cycle bit.
        let link = Trb { param: base, status: 0, control: TRB_LINK << 10 | LINK_TOGGLE };
        unsafe { write_volatile((base as *mut Trb).add(RING - 1), link) };
        Some(ProducerRing { base, index: 0, cycle: 1 })
    }

    /// Queues one TRB; returns its physical address.
    pub fn push(&mut self, mut trb: Trb) -> u64 {
        trb.control = (trb.control & !1) | self.cycle;
        let at = self.base + 16 * self.index as u64;
        unsafe { write_volatile(at as *mut Trb, trb) };
        self.index += 1;
        if self.index == RING - 1 {
            // Hand the link TRB over with the current cycle, then wrap.
            let link = (self.base + 16 * (RING as u64 - 1)) as *mut Trb;
            unsafe {
                let mut l = read_volatile(link);
                l.control = (l.control & !1) | self.cycle;
                write_volatile(link, l);
            }
            self.index = 0;
            self.cycle ^= 1;
        }
        at
    }
}

pub struct Controller {
    op: usize,
    db: usize,
    rt: usize,
    /// Context size in bytes: 32, or 64 when HCCPARAMS1.CSZ is set.
    pub ctx_size: u64,
    pub dcbaa: u64,
    pub commands: ProducerRing,
    events: u64,
    event_index: usize,
    event_cycle: u32,
}

// The controller's memory belongs to the driver; access goes through the lock.
unsafe impl Send for Controller {}

static CONTROLLER: SpinLock<Option<Controller>> = SpinLock::new(None);

fn r32(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}
fn w32(addr: usize, v: u32) {
    unsafe { write_volatile(addr as *mut u32, v) }
}
fn w64(addr: usize, v: u64) {
    // Two 32-bit writes, low half first: works with and without 64-bit access.
    w32(addr, v as u32);
    w32(addr + 4, (v >> 32) as u32);
}

/// Waits until `done` holds, polling for about `ms` milliseconds.
fn wait(ms: u64, mut done: impl FnMut() -> bool) -> bool {
    for _ in 0..ms * 100 {
        if done() {
            return true;
        }
        crate::arch::delay_us(10);
    }
    done()
}

impl Controller {
    pub fn port_status(&self, port: u32) -> u32 {
        r32(self.op + PORTSC + 0x10 * (port as usize - 1))
    }

    /// Writes a port's status register. Change bits are write-1-to-clear
    /// and writing 1 to "enabled" disables the port, so callers start from
    /// `port_status(p) & PORT_KEEP`.
    pub fn port_write(&self, port: u32, v: u32) {
        w32(self.op + PORTSC + 0x10 * (port as usize - 1), v)
    }

    pub fn doorbell(&self, slot: u32, target: u32) {
        fence(Ordering::SeqCst);
        w32(self.db + 4 * slot as usize, target);
    }

    /// The next event, if the controller has written one.
    pub fn next_event(&mut self) -> Option<Trb> {
        let at = self.events + 16 * self.event_index as u64;
        let trb = unsafe { read_volatile(at as *const Trb) };
        if trb.control & 1 != self.event_cycle {
            return None;
        }
        fence(Ordering::SeqCst);
        self.event_index += 1;
        if self.event_index == RING {
            self.event_index = 0;
            self.event_cycle ^= 1;
        }
        let next = self.events + 16 * self.event_index as u64;
        w64(self.rt + IR0 + ERDP, next | ERDP_BUSY);
        Some(trb)
    }

    /// Sends a command and waits for its completion event.
    pub fn command(&mut self, trb: Trb) -> Option<Trb> {
        let at = self.commands.push(trb);
        self.doorbell(0, 0);
        self.wait_event(|ev| ev.kind() == TRB_COMMAND_DONE && ev.param == at)
    }

    /// Waits (about a second at most) for an event that `wanted` accepts;
    /// other events on the way are dropped.
    pub fn wait_event(&mut self, wanted: impl Fn(&Trb) -> bool) -> Option<Trb> {
        for _ in 0..100_000 {
            while let Some(ev) = self.next_event() {
                if wanted(&ev) {
                    return Some(ev);
                }
            }
            crate::arch::delay_us(10);
        }
        None
    }
}

/// Finds the first xHCI controller: (bus 0 device, BAR address).
fn find() -> Option<(u8, u64)> {
    use crate::arch::{pci_read32, pci_write32};
    for dev in 0..32u8 {
        let id = pci_read32(0, dev, 0, 0);
        if id == 0xFFFF_FFFF {
            continue;
        }
        if pci_read32(0, dev, 0, 0x08) >> 8 != 0x0C_0330 {
            continue;
        }
        let lo = pci_read32(0, dev, 0, 0x10);
        if lo & 1 != 0 {
            return None;
        }
        let hi = if lo & 0x6 == 0x4 { pci_read32(0, dev, 0, 0x14) as u64 } else { 0 };
        // Memory decoding and bus mastering (the controller reads our rings).
        let cmd = pci_read32(0, dev, 0, 0x04);
        pci_write32(0, dev, 0, 0x04, cmd | 0x6);
        return Some((dev, hi << 32 | (lo & !0xF) as u64));
    }
    None
}

/// What `init` found.
pub struct Found {
    pub version: u16,
    pub ports: u32,
    pub connected: u32,
    pub scratchpads: u32,
    pub devices: alloc::vec::Vec<crate::usb::Device>,
}

/// Finds and starts the controller; `None` if there is none.
pub fn init() -> Option<Result<Found, &'static str>> {
    let (_, bar) = find()?;
    Some(start(bar as usize))
}

fn start(base: usize) -> Result<Found, &'static str> {
    // CAPLENGTH and HCIVERSION share the first word; some controllers only
    // answer 32-bit reads.
    let first = r32(base + CAPLENGTH);
    let caplen = (first & 0xFF) as usize;
    let version = (first >> 16) as u16;
    let params1 = r32(base + HCSPARAMS1);
    let params2 = r32(base + HCSPARAMS2);
    let op = base + caplen;
    let db = base + (r32(base + DBOFF) & !0x3) as usize;
    let rt = base + (r32(base + RTSOFF) & !0x1F) as usize;
    let max_slots = params1 & 0xFF;
    let ports = params1 >> 24;
    let scratchpads = ((params2 >> 21) & 0x1F) << 5 | (params2 >> 27);

    // Stop, then reset.
    w32(op + USBCMD, r32(op + USBCMD) & !CMD_RUN);
    if !wait(100, || r32(op + USBSTS) & STS_HALTED != 0) {
        return Err("controller does not halt");
    }
    w32(op + USBCMD, CMD_RESET);
    if !wait(500, || r32(op + USBCMD) & CMD_RESET == 0 && r32(op + USBSTS) & STS_NOT_READY == 0) {
        return Err("controller does not come out of reset");
    }

    // Device context base address array, with scratchpad pages in entry 0.
    let dcbaa = frame::alloc().ok_or("no memory")?;
    unsafe { (dcbaa as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
    if scratchpads > 0 {
        let array = frame::alloc().ok_or("no memory")?;
        unsafe { (array as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
        for i in 0..scratchpads as usize {
            let page = frame::alloc().ok_or("no memory")?;
            unsafe {
                (page as *mut u8).write_bytes(0, PAGE_SIZE as usize);
                (array as *mut u64).add(i).write_volatile(page);
            }
        }
        unsafe { (dcbaa as *mut u64).write_volatile(array) };
    }
    w32(op + CONFIG, max_slots);
    w64(op + DCBAAP, dcbaa);

    let commands = ProducerRing::new().ok_or("no memory")?;
    w64(op + CRCR, commands.base | 1);

    // One event ring segment, described by a one-entry table.
    let events = frame::alloc().ok_or("no memory")?;
    let erst = frame::alloc().ok_or("no memory")?;
    unsafe {
        (events as *mut u8).write_bytes(0, PAGE_SIZE as usize);
        (erst as *mut u8).write_bytes(0, PAGE_SIZE as usize);
        (erst as *mut u64).write_volatile(events);
        ((erst + 8) as *mut u32).write_volatile(RING as u32);
    }
    w32(rt + IR0 + ERSTSZ, 1);
    w64(rt + IR0 + ERDP, events);
    w64(rt + IR0 + ERSTBA, erst);

    w32(op + USBCMD, CMD_RUN);
    if !wait(100, || r32(op + USBSTS) & STS_HALTED == 0) {
        return Err("controller does not run");
    }

    let ctx_size = if r32(base + HCCPARAMS1) & (1 << 2) != 0 { 64 } else { 32 };
    let mut c = Controller { op, db, rt, ctx_size, dcbaa, commands, events, event_index: 0, event_cycle: 1 };
    // The rings work if a No-Op command completes.
    match c.command(Trb { param: 0, status: 0, control: TRB_NOOP_CMD << 10 }) {
        Some(ev) if ev.code() == COMPLETION_SUCCESS => {}
        Some(_) => return Err("No-Op command failed"),
        None => return Err("no answer to the No-Op command"),
    }
    // Ports with something plugged in (USB 3 devices come up enabled; USB 2
    // ones wait for a port reset, done when devices are enumerated).
    let connected = (1..=ports).filter(|&p| c.port_status(p) & PORT_CONNECTED != 0).count() as u32;
    let plugged: alloc::vec::Vec<u32> = (1..=ports).filter(|&p| c.port_status(p) & PORT_CONNECTED != 0).collect();
    let devices: alloc::vec::Vec<_> = plugged.into_iter().map(|p| crate::usb::enumerate(&mut c, p)).collect();
    crate::usb::start_reports(&mut c, &devices);
    *CONTROLLER.lock() = Some(c);
    Ok(Found { version, ports, connected, scratchpads, devices })
}

/// Runs `f` on the controller, if one was started.
pub fn with<R>(f: impl FnOnce(&mut Controller) -> R) -> Option<R> {
    CONTROLLER.lock().as_mut().map(f)
}
