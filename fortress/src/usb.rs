//! USB devices on an xHCI controller: setting one up (enumeration).
//!
//! For each root port with something plugged in: reset the port (USB 2
//! devices need it to be enabled), ask the controller for a device slot,
//! give the device an address with a context for its control endpoint,
//! then read its device and configuration descriptors over that endpoint.
//! A keyboard or mouse that speaks the HID boot protocol is configured and
//! switched to that protocol, so its reports have the simple fixed layout
//! a BIOS would read. Its reports then feed the keyboard input and the
//! pointer, as the PS/2 and virtio drivers do.
//!
//! A hub is set up like any device, told to the controller as a hub, and
//! then each of its ports is powered, reset and set up in turn, with the
//! route through the hubs in the device's slot. Low- and full-speed
//! devices behind a high-speed hub also name the hub's transaction
//! translator. Hubs are read once at boot; plugging in later comes with
//! interrupts. USB 3 hubs are counted but not walked yet: their keyboards
//! and mice show on the USB 2 side of the same hub.

use crate::mem::{frame, PAGE_SIZE};
use crate::xhci::{Controller, ProducerRing, Trb, COMPLETION_SUCCESS};
use crate::xhci::{PORT_CONNECTED, PORT_ENABLED, PORT_KEEP, PORT_RESET, PORT_RESET_CHANGE};

const TRB_SETUP: u32 = 2;
const TRB_DATA: u32 = 3;
const TRB_STATUS: u32 = 4;
const TRB_ENABLE_SLOT: u32 = 9;
const TRB_ADDRESS_DEVICE: u32 = 11;
const TRB_EVALUATE_CONTEXT: u32 = 13;
const TRB_TRANSFER_DONE: u32 = 32;
const COMPLETION_SHORT: u32 = 13;
const IDT: u32 = 1 << 6;
const IOC: u32 = 1 << 5;
const DIR_IN: u32 = 1 << 16;

/// What a device is to Neo, from its interface descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Keyboard,
    Mouse,
    /// A hub with this many ports.
    Hub(u8),
    Other,
}

/// A device that was set up (or why not).
#[derive(Debug)]
pub struct Device {
    /// The root port, then the hub port at each tier: "3" or "3.2".
    pub path: alloc::string::String,
    pub speed: u32,
    pub slot: u32,
    pub vendor: u16,
    pub product: u16,
    pub kind: Kind,
    /// The boot interface's interrupt IN endpoint: (address, packet size, interval).
    pub endpoint: Option<(u8, u16, u8)>,
    pub error: Option<&'static str>,
    /// The first three words of its slot context: route and speed, root
    /// port, transaction translator. Configure Endpoint repeats them.
    slot_ctx: [u32; 3],
}

impl Device {
    pub fn speed_name(&self) -> &'static str {
        match self.speed {
            1 => "full speed",
            2 => "low speed",
            3 => "high speed",
            4 | 5 => "SuperSpeed",
            _ => "unknown speed",
        }
    }
}

fn zeroed_page() -> Option<u64> {
    let p = frame::alloc()?;
    unsafe { (p as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
    Some(p)
}

/// Resets a USB 2 port so the device behind it is enabled; USB 3 ports
/// enable themselves.
fn reset_port(c: &Controller, port: u32) -> Result<u32, &'static str> {
    let ps = c.port_status(port);
    if ps & PORT_ENABLED == 0 {
        c.port_write(port, (ps & PORT_KEEP) | PORT_RESET);
        let mut done = false;
        for _ in 0..50_000 {
            if c.port_status(port) & PORT_RESET_CHANGE != 0 {
                done = true;
                break;
            }
            crate::arch::delay_us(10);
        }
        if !done {
            return Err("port reset did not finish");
        }
        c.port_write(port, (c.port_status(port) & PORT_KEEP) | PORT_RESET_CHANGE);
        // A device needs 10 ms after reset before it answers.
        crate::arch::delay_us(10_000);
    }
    let ps = c.port_status(port);
    if ps & (PORT_CONNECTED | PORT_ENABLED) != PORT_CONNECTED | PORT_ENABLED {
        return Err("port not enabled after reset");
    }
    Ok((ps >> 10) & 0xF)
}

/// A control transfer on endpoint 0; `data` is the buffer page for IN data.
/// Returns the length asked for (descriptors say their own length).
fn control(
    c: &mut Controller,
    ring: &mut ProducerRing,
    slot: u32,
    setup: (u8, u8, u16, u16, u16),
    data: u64,
) -> Result<usize, &'static str> {
    let (request_type, request, value, index, length) = setup;
    let packet = request_type as u64
        | (request as u64) << 8
        | (value as u64) << 16
        | (index as u64) << 32
        | (length as u64) << 48;
    let into = request_type & 0x80 != 0;
    let transfer_type = if length == 0 {
        0
    } else if into {
        3
    } else {
        2
    };
    ring.push(Trb { param: packet, status: 8, control: TRB_SETUP << 10 | IDT | transfer_type << 16 });
    if length > 0 {
        ring.push(Trb { param: data, status: length as u32, control: TRB_DATA << 10 | if into { DIR_IN } else { 0 } });
    }
    // The status stage goes the other way from the data (IN when there is none).
    let status_in = length == 0 || !into;
    let at =
        ring.push(Trb { param: 0, status: 0, control: TRB_STATUS << 10 | IOC | if status_in { DIR_IN } else { 0 } });
    c.doorbell(slot, 1);
    let ev =
        c.wait_event(|ev| ev.kind() == TRB_TRANSFER_DONE && ev.param == at).ok_or("no answer to a control transfer")?;
    match ev.code() {
        COMPLETION_SUCCESS | COMPLETION_SHORT => Ok(length as usize),
        _ => Err("control transfer failed"),
    }
}

/// Where a device is plugged in.
struct Place {
    root_port: u32,
    route: u32,
    depth: u32,
    speed: u32,
    /// The high-speed hub (slot, port) whose translator a low- or
    /// full-speed device talks through; (0, 0) when there is none.
    tt: (u32, u32),
    path: alloc::string::String,
}

/// Sets up the device on root port `port`, and everything behind it when
/// it is a hub. Never fails as a whole: problems end up in `error`, so the
/// boot log can say what happened on each port.
pub fn enumerate(c: &mut Controller, port: u32) -> alloc::vec::Vec<Device> {
    let mut out = alloc::vec::Vec::new();
    let path = alloc::format!("{}", port);
    match reset_port(c, port) {
        Ok(speed) => attach(c, Place { root_port: port, route: 0, depth: 0, speed, tt: (0, 0), path }, &mut out),
        Err(e) => {
            let mut d = Device::new(path, 0);
            d.error = Some(e);
            out.push(d);
        }
    }
    out
}

impl Device {
    fn new(path: alloc::string::String, speed: u32) -> Device {
        Device {
            path,
            speed,
            slot: 0,
            vendor: 0,
            product: 0,
            kind: Kind::Other,
            endpoint: None,
            error: None,
            slot_ctx: [0; 3],
        }
    }
}

fn attach(c: &mut Controller, place: Place, out: &mut alloc::vec::Vec<Device>) {
    let at = out.len();
    out.push(Device::new(place.path.clone(), place.speed));
    if let Err(e) = setup(c, &place, at, out) {
        out[at].error = Some(e);
    }
}

fn setup(c: &mut Controller, place: &Place, at: usize, out: &mut alloc::vec::Vec<Device>) -> Result<(), &'static str> {
    let ev =
        c.command(Trb { param: 0, status: 0, control: TRB_ENABLE_SLOT << 10 }).ok_or("no answer to Enable Slot")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("no free device slot");
    }
    let slot = ev.control >> 24;
    out[at].slot = slot;

    // The device's output context, where the controller keeps its state.
    let output = zeroed_page().ok_or("no memory")?;
    unsafe { (c.dcbaa as *mut u64).add(slot as usize).write_volatile(output) };

    // Input context: control context, slot context, endpoint 0 context.
    let input = zeroed_page().ok_or("no memory")?;
    let mut ring = ProducerRing::new().ok_or("no memory")?;
    let cs = c.ctx_size;
    let speed = place.speed;
    let mut packet0: u32 = match speed {
        3 => 64,
        4 | 5 => 512,
        _ => 8,
    };
    let slot_ctx = [place.route | speed << 20, place.root_port << 16, place.tt.0 | place.tt.1 << 8];
    out[at].slot_ctx = slot_ctx;
    let w = |ctx: u64, dword: u64, v: u32| unsafe { ((input + ctx * cs + 4 * dword) as *mut u32).write_volatile(v) };
    w(0, 1, 0b11); // add the slot and endpoint 0
    w(1, 0, slot_ctx[0] | 1 << 27); // route, speed, one context entry
    w(1, 1, slot_ctx[1]); // root hub port
    w(1, 2, slot_ctx[2]); // transaction translator
    w(2, 1, 3 << 1 | 4 << 3 | packet0 << 16); // 3 retries, control endpoint, packet size
    w(2, 2, ring.base as u32 | 1); // dequeue pointer, cycle 1
    w(2, 3, (ring.base >> 32) as u32);
    w(2, 4, 8); // average TRB length
    let ev = c
        .command(Trb { param: input, status: 0, control: TRB_ADDRESS_DEVICE << 10 | slot << 24 })
        .ok_or("no answer to Address Device")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("the device took no address");
    }

    let buf = zeroed_page().ok_or("no memory")?;
    // A full-speed device may take 8 to 64 bytes per packet on endpoint 0;
    // the first 8 bytes of its descriptor say which, and the controller is
    // told before anything longer is read (Evaluate Context).
    if speed == 1 {
        control(c, &mut ring, slot, (0x80, 6, 0x0100, 0, 8), buf)?;
        let size = unsafe { (buf as *const u8).add(7).read_volatile() } as u32;
        if size != packet0 && matches!(size, 8 | 16 | 32 | 64) {
            packet0 = size;
            w(0, 1, 0b10); // endpoint 0 only
            w(2, 1, 3 << 1 | 4 << 3 | packet0 << 16);
            let ev = c
                .command(Trb { param: input, status: 0, control: TRB_EVALUATE_CONTEXT << 10 | slot << 24 })
                .ok_or("no answer to Evaluate Context")?;
            if ev.code() != COMPLETION_SUCCESS {
                return Err("endpoint 0 packet size not taken");
            }
        }
    }
    let d = &mut out[at];
    // Device descriptor.
    let n = control(c, &mut ring, slot, (0x80, 6, 0x0100, 0, 18), buf)?;
    if n < 12 {
        return Err("short device descriptor");
    }
    let desc = unsafe { core::slice::from_raw_parts(buf as *const u8, 18) };
    d.vendor = u16::from_le_bytes([desc[8], desc[9]]);
    d.product = u16::from_le_bytes([desc[10], desc[11]]);
    let hub = desc[4] == 9;

    // Configuration descriptor: its header first, then the whole thing.
    control(c, &mut ring, slot, (0x80, 6, 0x0200, 0, 9), buf)?;
    let total = unsafe { ((buf + 2) as *const u16).read_unaligned() }.min(PAGE_SIZE as u16);
    let n = control(c, &mut ring, slot, (0x80, 6, 0x0200, 0, total), buf)?;
    let cfg = unsafe { core::slice::from_raw_parts(buf as *const u8, n) };
    let config_value = cfg[5];

    // Find a HID boot interface and its interrupt IN endpoint.
    let mut i = 0;
    let mut interface = None;
    while i + 2 <= cfg.len() && cfg[i] >= 2 {
        let (len, kind) = (cfg[i] as usize, cfg[i + 1]);
        if i + len > cfg.len() {
            break;
        }
        match kind {
            4 if len >= 9 && interface.is_none() && cfg[i + 5] == 3 && cfg[i + 6] == 1 => {
                d.kind = match cfg[i + 7] {
                    1 => Kind::Keyboard,
                    2 => Kind::Mouse,
                    _ => Kind::Other,
                };
                interface = Some(cfg[i + 2]);
            }
            5 if len >= 7 && interface.is_some() && d.endpoint.is_none() => {
                let address = cfg[i + 2];
                if address & 0x80 != 0 && cfg[i + 3] & 3 == 3 {
                    let size = u16::from_le_bytes([cfg[i + 4], cfg[i + 5]]) & 0x7FF;
                    d.endpoint = Some((address, size, cfg[i + 6]));
                }
            }
            _ => {}
        }
        i += len;
    }

    control(c, &mut ring, slot, (0x00, 9, config_value as u16, 0, 0), buf)?;
    if hub {
        d.kind = Kind::Hub(0);
        d.endpoint = None;
        if speed >= 4 {
            return Ok(()); // a USB 3 hub: its ports are not walked yet
        }
        return walk_hub(c, &mut ring, place, at, out, input, buf);
    }
    if let (Some(iface), Kind::Keyboard | Kind::Mouse) = (interface, d.kind) {
        // SET_PROTOCOL(boot); SET_IDLE(0) so a keyboard reports only changes.
        control(c, &mut ring, slot, (0x21, 0x0B, 0, iface as u16, 0), buf)?;
        if d.kind == Kind::Keyboard {
            let _ = control(c, &mut ring, slot, (0x21, 0x0A, 0, iface as u16, 0), buf);
        }
    }
    Ok(())
}

// Hub class requests and port features (USB 2.0 chapter 11).
const PORT_POWER: u16 = 8;
const PORT_RESET_FEATURE: u16 = 4;
const C_PORT_CONNECTION: u16 = 16;
const C_PORT_RESET: u16 = 20;

/// A hub port's status and change bits.
fn hub_port_status(
    c: &mut Controller,
    ring: &mut ProducerRing,
    slot: u32,
    port: u16,
    buf: u64,
) -> Result<(u16, u16), &'static str> {
    control(c, ring, slot, (0xA3, 0, 0, port, 4), buf)?;
    let v = unsafe { (buf as *const u32).read_volatile() };
    Ok((v as u16, (v >> 16) as u16))
}

/// Tells the controller the device is a hub, then powers its ports and
/// sets up what is plugged into each, one port at a time (a new device
/// answers on address 0 until it has its own).
fn walk_hub(
    c: &mut Controller,
    ring: &mut ProducerRing,
    place: &Place,
    at: usize,
    out: &mut alloc::vec::Vec<Device>,
    input: u64,
    buf: u64,
) -> Result<(), &'static str> {
    let slot = out[at].slot;
    control(c, ring, slot, (0xA0, 6, 0x2900, 0, 9), buf)?;
    let desc = unsafe { core::slice::from_raw_parts(buf as *const u8, 9) };
    let ports = desc[2].min(15);
    let think = (u16::from_le_bytes([desc[3], desc[4]]) >> 5 & 3) as u32;
    let power_good_ms = desc[5] as u64 * 2;
    out[at].kind = Kind::Hub(ports);

    // Slot context: a hub, its port count and (high speed) TT think time.
    let cs = c.ctx_size;
    let mut ctx = out[at].slot_ctx;
    ctx[0] |= 1 << 26;
    ctx[1] |= (ports as u32) << 24;
    if place.speed == 3 {
        ctx[2] |= think << 16;
    }
    out[at].slot_ctx = ctx;
    unsafe {
        (input as *mut u8).write_bytes(0, PAGE_SIZE as usize);
        let w = |ctx: u64, dword: u64, v: u32| ((input + ctx * cs + 4 * dword) as *mut u32).write_volatile(v);
        w(0, 1, 1); // the slot only
        w(1, 0, ctx[0] | 1 << 27);
        w(1, 1, ctx[1]);
        w(1, 2, ctx[2]);
    }
    let ev = c
        .command(Trb { param: input, status: 0, control: TRB_CONFIGURE_ENDPOINT << 10 | slot << 24 })
        .ok_or("no answer to Configure Endpoint")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("the controller did not take the hub");
    }
    if place.depth >= 4 {
        return Err("hubs nested too deep");
    }

    for port in 1..=ports as u16 {
        control(c, ring, slot, (0x23, 3, PORT_POWER, port, 0), buf)?;
    }
    // Power good, then time for devices to connect (debounce).
    crate::arch::delay_us((power_good_ms + 100) * 1000);

    for port in 1..=ports as u16 {
        let (status, _) = hub_port_status(c, ring, slot, port, buf)?;
        if status & 1 == 0 {
            continue;
        }
        let path = alloc::format!("{}.{}", place.path, port);
        let speed = match reset_hub_port(c, ring, slot, port, buf) {
            Ok(speed) => speed,
            Err(e) => {
                let mut d = Device::new(path, 0);
                d.error = Some(e);
                out.push(d);
                continue;
            }
        };
        // Low and full speed behind a high-speed hub go through its translator.
        let tt = if place.speed == 3 && speed <= 2 { (slot, port as u32) } else { place.tt };
        let child = Place {
            root_port: place.root_port,
            route: place.route | (port as u32) << (4 * place.depth),
            depth: place.depth + 1,
            speed,
            tt,
            path,
        };
        attach(c, child, out);
    }
    Ok(())
}

/// Resets one hub port; returns the xHCI speed of the device on it.
fn reset_hub_port(
    c: &mut Controller,
    ring: &mut ProducerRing,
    slot: u32,
    port: u16,
    buf: u64,
) -> Result<u32, &'static str> {
    control(c, ring, slot, (0x23, 3, PORT_RESET_FEATURE, port, 0), buf)?;
    let mut done = false;
    for _ in 0..50 {
        crate::arch::delay_us(10_000);
        let (_, change) = hub_port_status(c, ring, slot, port, buf)?;
        if change & 1 << 4 != 0 {
            done = true;
            break;
        }
    }
    if !done {
        return Err("hub port reset did not finish");
    }
    control(c, ring, slot, (0x23, 1, C_PORT_RESET, port, 0), buf)?;
    control(c, ring, slot, (0x23, 1, C_PORT_CONNECTION, port, 0), buf)?;
    crate::arch::delay_us(10_000);
    let (status, _) = hub_port_status(c, ring, slot, port, buf)?;
    if status & 0b11 != 0b11 {
        return Err("hub port not enabled after reset");
    }
    Ok(if status & 1 << 9 != 0 {
        2 // low speed
    } else if status & 1 << 10 != 0 {
        3 // high speed
    } else {
        1 // full speed
    })
}

// ---- Reading boot reports (keyboards and mice) ----

use crate::sync::SpinLock;
use core::sync::atomic::{AtomicU64, Ordering};
use neo_keys::Keys;

const TRB_NORMAL: u32 = 1;
const TRB_CONFIGURE_ENDPOINT: u32 = 12;
/// Interrupt on a short packet: a report shorter than the buffer.
const ISP: u32 = 1 << 2;
/// Transfers kept queued on each endpoint.
const QUEUED: usize = 8;

/// Keys pressed and mouse reports read, for the boot log and tests.
pub static KEYS: AtomicU64 = AtomicU64::new(0);
pub static MOUSE_REPORTS: AtomicU64 = AtomicU64::new(0);

struct Reader {
    slot: u32,
    dci: u32,
    kind: Kind,
    ring: ProducerRing,
    size: u16,
    previous: [u8; 8],
    keys: Keys,
}

// The rings belong to the driver; access goes through the lock.
unsafe impl Send for Reader {}

static READERS: SpinLock<alloc::vec::Vec<Reader>> = SpinLock::new(alloc::vec::Vec::new());

/// The xHCI interval for an interrupt endpoint (2^n x 125 us).
fn interval(speed: u32, b_interval: u8) -> u32 {
    let b = b_interval.max(1) as u32;
    match speed {
        3..=5 => (b - 1).min(15),
        // Full and low speed count in 1 ms frames: 8 x 125 us each.
        _ => (31 - (b * 8).leading_zeros()).clamp(3, 10),
    }
}

/// Starts reading reports from every boot keyboard and mouse: adds the
/// interrupt IN endpoint to the device's slot (Configure Endpoint) and
/// keeps transfers queued on it.
pub fn start_reports(c: &mut Controller, devices: &[Device]) -> usize {
    let mut started = 0;
    for d in devices.iter().filter(|d| d.error.is_none() && matches!(d.kind, Kind::Keyboard | Kind::Mouse)) {
        let Some((address, size, b_interval)) = d.endpoint else { continue };
        match configure(c, d, address, size, b_interval) {
            Ok(r) => {
                READERS.lock().push(r);
                started += 1;
            }
            Err(e) => crate::kprintln!("  USB port {}: reports not started: {}", d.path, e),
        }
    }
    started
}

fn configure(c: &mut Controller, d: &Device, address: u8, size: u16, b_interval: u8) -> Result<Reader, &'static str> {
    let dci = (address & 0xF) as u32 * 2 + 1;
    let ring = ProducerRing::new().ok_or("no memory")?;
    let input = zeroed_page().ok_or("no memory")?;
    let cs = c.ctx_size;
    let size = size.clamp(1, 64);
    unsafe {
        let w = |ctx: u64, dword: u64, v: u32| ((input + ctx * cs + 4 * dword) as *mut u32).write_volatile(v);
        w(0, 1, 1 | 1 << dci); // the slot and the new endpoint
        w(1, 0, d.slot_ctx[0] | dci << 27);
        w(1, 1, d.slot_ctx[1]);
        w(1, 2, d.slot_ctx[2]);
        let ep = 1 + dci as u64;
        w(ep, 0, interval(d.speed, b_interval) << 16);
        w(ep, 1, 3 << 1 | 7 << 3 | (size as u32) << 16); // 3 retries, interrupt IN
        w(ep, 2, ring.base as u32 | 1);
        w(ep, 3, (ring.base >> 32) as u32);
        w(ep, 4, size as u32 | (size as u32) << 16); // average TRB length, max ESIT payload
    }
    let ev = c
        .command(Trb { param: input, status: 0, control: TRB_CONFIGURE_ENDPOINT << 10 | d.slot << 24 })
        .ok_or("no answer to Configure Endpoint")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("Configure Endpoint failed");
    }
    // Report buffers: 64 bytes each in one page.
    let buffers = zeroed_page().ok_or("no memory")?;
    let mut r = Reader { slot: d.slot, dci, kind: d.kind, ring, size, previous: [0; 8], keys: Keys::default() };
    for i in 0..QUEUED as u64 {
        r.queue(buffers + 64 * i);
    }
    c.doorbell(d.slot, dci);
    Ok(r)
}

impl Reader {
    fn queue(&mut self, buffer: u64) {
        self.ring.push(Trb { param: buffer, status: self.size as u32, control: TRB_NORMAL << 10 | IOC | ISP });
    }

    fn report(&mut self, data: &[u8]) {
        match self.kind {
            Kind::Keyboard if data.len() >= 8 => self.keyboard(data),
            Kind::Mouse if data.len() >= 3 => {
                MOUSE_REPORTS.fetch_add(1, Ordering::Relaxed);
                crate::pointer::moved(data[1] as i8 as i64, data[2] as i8 as i64, (data[0] & 7) as u64);
            }
            _ => {}
        }
    }

    /// A boot keyboard report: modifier bits, a reserved byte, then up to
    /// six keys held down. Keys are compared with the previous report.
    fn keyboard(&mut self, data: &[u8]) {
        const MODIFIERS: [u16; 8] = [29, 42, 56, 125, 97, 54, 100, 126];
        let (old, new) = (self.previous, [data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        // Phantom state (every slot 1): too many keys at once; ignore it.
        if new[2..].iter().all(|&k| k == 1) {
            return;
        }
        for (bit, &code) in MODIFIERS.iter().enumerate() {
            let (was, is) = (old[0] >> bit & 1 != 0, new[0] >> bit & 1 != 0);
            if was != is {
                self.key(code, is);
            }
        }
        for &u in old[2..].iter().filter(|&&u| u > 3 && !new[2..].contains(&u)) {
            if let Some(code) = hid_to_key(u) {
                self.key(code, false);
            }
        }
        for &u in new[2..].iter().filter(|&&u| u > 3 && !old[2..].contains(&u)) {
            if let Some(code) = hid_to_key(u) {
                KEYS.fetch_add(1, Ordering::Relaxed);
                self.key(code, true);
            }
        }
        self.previous = new;
    }

    /// One key event through neo-keys, as the other keyboard drivers do.
    fn key(&mut self, code: u16, down: bool) {
        let now = crate::input::layout();
        self.keys.layout = neo_keys::Layout::from_number(now);
        let mut out = [0; 4];
        let n = self.keys.key_bytes(code, down, &mut out);
        if self.keys.layout.number() != now {
            crate::input::set_layout(self.keys.layout.number());
        }
        for &b in &out[..n] {
            crate::input::push(b);
        }
    }
}

/// HID keyboard usage to the key codes neo-keys reads (the PC set that
/// Linux also uses).
fn hid_to_key(u: u8) -> Option<u16> {
    const LETTERS: [u16; 26] =
        [30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47, 17, 45, 21, 44];
    Some(match u {
        0x04..=0x1D => LETTERS[(u - 0x04) as usize],
        0x1E..=0x27 => (u - 0x1E) as u16 + 2,
        0x28 => 28,
        0x29 => 1,
        0x2A => 14,
        0x2B => 15,
        0x2C => 57,
        0x2D => 12,
        0x2E => 13,
        0x2F => 26,
        0x30 => 27,
        0x31 | 0x32 => 43,
        0x33 => 39,
        0x34 => 40,
        0x35 => 41,
        0x36 => 51,
        0x37 => 52,
        0x38 => 53,
        0x39 => 58,
        0x3A..=0x43 => (u - 0x3A) as u16 + 59,
        0x44 => 87,
        0x45 => 88,
        0x4A => 102,
        0x4B => 104,
        0x4C => 111,
        0x4D => 107,
        0x4E => 109,
        0x4F => 106,
        0x50 => 105,
        0x51 => 108,
        0x52 => 103,
        0x64 => 86,
        _ => return None,
    })
}

/// Handles finished transfers: hands each report on and queues its buffer
/// again. Called by the USB thread a hundred times a second.
pub fn poll() {
    crate::xhci::with(|c| {
        let mut readers = READERS.lock();
        let mut rang = alloc::vec::Vec::new();
        while let Some(ev) = c.next_event() {
            if ev.kind() != TRB_TRANSFER_DONE {
                continue;
            }
            let (slot, dci) = (ev.control >> 24, (ev.control >> 16) & 0x1F);
            let Some(r) = readers.iter_mut().find(|r| r.slot == slot && r.dci == dci) else { continue };
            // The finished TRB holds its buffer; the event, what was not filled.
            let trb = unsafe { (ev.param as *const Trb).read_volatile() };
            let got = (r.size as usize).saturating_sub((ev.status & 0xFF_FFFF) as usize);
            if matches!(ev.code(), COMPLETION_SUCCESS | COMPLETION_SHORT) {
                let data = unsafe { core::slice::from_raw_parts(trb.param as *const u8, got) };
                r.report(data);
            }
            r.queue(trb.param);
            rang.push((slot, dci));
        }
        for (slot, dci) in rang {
            c.doorbell(slot, dci);
        }
    });
}

/// The USB thread: polls for reports while there are devices to read.
pub fn thread() {
    loop {
        poll();
        crate::sched::sleep(1);
    }
}

/// True if a keyboard or mouse is being read.
pub fn reading() -> bool {
    !READERS.lock().is_empty()
}
