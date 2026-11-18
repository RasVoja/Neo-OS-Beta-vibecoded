//! Virtio keyboards and pointers (virtio-input), legacy MMIO interface, by
//! interrupt.
//!
//! These are the keyboard and the tablet or mouse of the AArch64 `virt`
//! machine (`-device virtio-keyboard-device`, `virtio-tablet-device`,
//! `virtio-mouse-device`). A device writes 8-byte input events (type,
//! code, value) into buffers the driver has offered in its event queue,
//! and raises its interrupt. The handler turns key presses into characters
//! (see the neo-keys library) for the console input, and pointer events into moves
//! (see pointer.rs), then offers the buffers again.
//!
//! Like the disk driver it lives in the kernel as a bootstrap. On x86_64
//! the keyboard driver already runs in user mode (bin/keyboard); this one
//! follows once user-mode drivers can be given device memory and DMA
//! buffers as capabilities.

use crate::mem::{frame, PAGE_SIZE};
use crate::sync::SpinLock;
use core::sync::atomic::{fence, AtomicU64, Ordering};
use neo_keys::Keys;

const STATUS_ACK: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const DESC_WRITE: u16 = 2;

// Legacy virtio-mmio registers.
const DEVICE_ID: usize = 0x008;
const HOST_FEATURES: usize = 0x010;
const GUEST_FEATURES: usize = 0x020;
const GUEST_PAGE_SIZE: usize = 0x028;
const QUEUE_SEL: usize = 0x030;
const QUEUE_NUM_MAX: usize = 0x034;
const QUEUE_NUM: usize = 0x038;
const QUEUE_ALIGN: usize = 0x03C;
const QUEUE_PFN: usize = 0x040;
const QUEUE_NOTIFY: usize = 0x050;
const INTERRUPT_STATUS: usize = 0x060;
const INTERRUPT_ACK: usize = 0x064;
const STATUS: usize = 0x070;

const SLOTS: usize = 32;
const SLOT_BASE: usize = 0x0A00_0000;
/// The `virt` machine wires slot n to shared interrupt 16 + n (INTID 48 + n).
const FIRST_INTID: u32 = 48;
const CONFIG_SELECT: usize = 0x100;
const CONFIG_SUBSEL: usize = 0x101;
const CONFIG_SIZE: usize = 0x102;
const CFG_EV_BITS: u8 = 0x11;
const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_REL: u16 = 2;
const EV_ABS: u16 = 3;
const BTN_LEFT: u16 = 0x110;
/// A tablet reports positions in 0..=0x7FFF.
const ABS_RANGE: u64 = 0x8000;
/// At most this many input devices.
const MAX_DEVICES: usize = 4;
const QUEUE: u16 = 64;

/// Keys pressed so far (for the boot log and tests).
pub static KEYS: AtomicU64 = AtomicU64::new(0);

#[repr(C)]
#[derive(Clone, Copy)]
struct Desc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Event {
    kind: u16,
    code: u16,
    value: u32,
}

struct Device {
    base: usize,
    intid: u32,
    ring: u64,
    events: u64,
    last_used: u16,
    keys: Keys,
    /// Pointer events wait for the end of their report (EV_SYN).
    abs: (Option<u64>, Option<u64>),
    rel: (i64, i64),
    buttons: u64,
    touched: bool,
}

// The queue memory belongs to the driver; access goes through the lock.
unsafe impl Send for Device {}

static DEVICES: SpinLock<[Option<Device>; MAX_DEVICES]> = SpinLock::new([const { None }; MAX_DEVICES]);

fn reg(base: usize, r: usize) -> u32 {
    unsafe { ((base + r) as *const u32).read_volatile() }
}

fn set(base: usize, r: usize, v: u32) {
    unsafe { ((base + r) as *mut u32).write_volatile(v) }
}

fn used_offset() -> u64 {
    (16 * QUEUE as u64 + 6 + 2 * QUEUE as u64).next_multiple_of(PAGE_SIZE)
}

impl Device {
    fn avail(&self) -> *mut u16 {
        (self.ring + 16 * QUEUE as u64) as *mut u16
    }

    fn used(&self) -> *const u16 {
        (self.ring + used_offset()) as *const u16
    }

    /// Puts descriptor `id` back in the available ring.
    fn offer(&mut self, id: u16) {
        unsafe {
            let avail = self.avail();
            let idx = avail.add(1).read_volatile();
            avail.add(2 + (idx % QUEUE) as usize).write_volatile(id);
            fence(Ordering::SeqCst);
            avail.add(1).write_volatile(idx.wrapping_add(1));
        }
    }

    /// Takes every event the device has written since last time.
    fn drain(&mut self) {
        let mut offered = false;
        loop {
            let used_idx = unsafe { self.used().add(1).read_volatile() };
            if used_idx == self.last_used {
                break;
            }
            fence(Ordering::SeqCst);
            // A used element is (id: u32, len: u32), after flags and idx.
            let slot = (self.last_used % QUEUE) as usize;
            let id = unsafe { (self.used().add(2) as *const u32).add(2 * slot).read_volatile() } as u16;
            self.last_used = self.last_used.wrapping_add(1);
            let ev = unsafe { (self.events as *const Event).add(id as usize).read_volatile() };
            // Value 1 is a press and 2 a repeat; 0 is a release.
            match ev.kind {
                EV_KEY if (BTN_LEFT..BTN_LEFT + 3).contains(&ev.code) => {
                    // Left, right, middle: buttons bit 0, 1, 2.
                    let bit = 1 << (ev.code - BTN_LEFT);
                    self.buttons = if ev.value != 0 { self.buttons | bit } else { self.buttons & !bit };
                    self.touched = true;
                }
                EV_KEY => {
                    if ev.value == 1 {
                        KEYS.fetch_add(1, Ordering::Relaxed);
                    }
                    // One key can type more than one byte (č is two, an arrow three).
                    // The layout is kept in `input` (the Bar can switch it).
                    let now = crate::input::layout();
                    self.keys.layout = neo_keys::Layout::from_number(now);
                    let mut out = [0; 4];
                    let n = self.keys.key_bytes(ev.code, ev.value != 0, &mut out);
                    if self.keys.layout.number() != now {
                        crate::input::set_layout(self.keys.layout.number());
                    }
                    for &b in &out[..n] {
                        crate::input::push(b);
                    }
                }
                EV_ABS if ev.code == 0 => (self.abs.0, self.touched) = (Some(ev.value as u64), true),
                EV_ABS if ev.code == 1 => (self.abs.1, self.touched) = (Some(ev.value as u64), true),
                EV_REL if ev.code == 0 => (self.rel.0, self.touched) = (self.rel.0 + ev.value as i32 as i64, true),
                EV_REL if ev.code == 1 => (self.rel.1, self.touched) = (self.rel.1 + ev.value as i32 as i64, true),
                EV_SYN if self.touched => {
                    if self.abs != (None, None) {
                        crate::pointer::absolute(self.abs.0, self.abs.1, self.buttons, ABS_RANGE);
                    } else {
                        crate::pointer::moved(self.rel.0, self.rel.1, self.buttons);
                    }
                    (self.abs, self.rel, self.touched) = ((None, None), (0, 0), false);
                }
                _ => {}
            }
            self.offer(id);
            offered = true;
        }
        if offered {
            fence(Ordering::SeqCst);
            set(self.base, QUEUE_NOTIFY, 0);
        }
    }
}

fn setup(base: usize, intid: u32) -> Result<Device, &'static str> {
    set(base, STATUS, 0);
    set(base, STATUS, STATUS_ACK);
    set(base, STATUS, STATUS_ACK | STATUS_DRIVER);
    let _ = reg(base, HOST_FEATURES);
    set(base, GUEST_FEATURES, 0);
    set(base, GUEST_PAGE_SIZE, PAGE_SIZE as u32);
    set(base, QUEUE_SEL, 0);
    if (reg(base, QUEUE_NUM_MAX) as u16) < QUEUE {
        return Err("event queue too small");
    }
    set(base, QUEUE_NUM, QUEUE as u32);
    set(base, QUEUE_ALIGN, PAGE_SIZE as u32);
    let pages = (used_offset() + (6 + 8 * QUEUE as u64).next_multiple_of(PAGE_SIZE)) / PAGE_SIZE;
    let ring = frame::alloc_contiguous(pages as usize).ok_or("no memory for the queue")?;
    unsafe { (ring as *mut u8).write_bytes(0, (pages * PAGE_SIZE) as usize) };
    let events = frame::alloc().ok_or("no memory for events")?;
    let mut k = Device {
        base,
        intid,
        ring,
        events,
        last_used: 0,
        keys: Keys::default(),
        abs: (None, None),
        rel: (0, 0),
        buttons: 0,
        touched: false,
    };
    for i in 0..QUEUE {
        unsafe {
            (ring as *mut Desc).add(i as usize).write(Desc {
                addr: events + 8 * i as u64,
                len: 8,
                flags: DESC_WRITE,
                next: 0,
            });
        }
        k.offer(i);
    }
    set(base, QUEUE_PFN, (ring / PAGE_SIZE) as u32);
    set(base, STATUS, STATUS_ACK | STATUS_DRIVER | STATUS_DRIVER_OK);
    fence(Ordering::SeqCst);
    set(base, QUEUE_NOTIFY, 0);
    Ok(k)
}

/// True if the device reports pointer movement (absolute or relative).
fn is_pointer(base: usize) -> bool {
    let has = |kind: u16| unsafe {
        ((base + CONFIG_SELECT) as *mut u8).write_volatile(CFG_EV_BITS);
        ((base + CONFIG_SUBSEL) as *mut u8).write_volatile(kind as u8);
        ((base + CONFIG_SIZE) as *const u8).read_volatile() > 0
    };
    has(EV_ABS) || has(EV_REL)
}

/// What `init` found in one slot.
pub struct Found {
    pub intid: u32,
    pub pointer: bool,
    pub result: Result<(), &'static str>,
}

/// Looks for virtio input devices and starts them.
pub fn init() -> alloc::vec::Vec<Found> {
    let mut found = alloc::vec::Vec::new();
    let mut devices = DEVICES.lock();
    for slot in 0..SLOTS {
        let base = SLOT_BASE + slot * 0x200;
        // Magic "virt", legacy version 1, device type 18 (input).
        if reg(base, 0) != 0x7472_6976 || reg(base, 4) != 1 || reg(base, DEVICE_ID) != 18 {
            continue;
        }
        let Some(free) = devices.iter().position(|d| d.is_none()) else { break };
        let intid = FIRST_INTID + slot as u32;
        let pointer = is_pointer(base);
        let result = setup(base, intid).map(|d| devices[free] = Some(d));
        found.push(Found { intid, pointer, result });
    }
    found
}

/// Handles interrupt `intid` if it belongs to an input device; returns
/// whether it did.
pub fn irq(intid: u32) -> bool {
    let mut devices = DEVICES.lock();
    let Some(k) = devices.iter_mut().flatten().find(|d| d.intid == intid) else { return false };
    let pending = reg(k.base, INTERRUPT_STATUS);
    set(k.base, INTERRUPT_ACK, pending);
    k.drain();
    true
}
