//! USB devices on an xHCI controller: setting one up (enumeration).
//!
//! For each root port with something plugged in: reset the port (USB 2
//! devices need it to be enabled), ask the controller for a device slot,
//! give the device an address with a context for its control endpoint,
//! then read its device and configuration descriptors over that endpoint.
//! A keyboard or mouse that speaks the HID boot protocol is configured and
//! switched to that protocol, so its reports have the simple fixed layout
//! a BIOS would read. Reading the reports comes next.

use crate::mem::{frame, PAGE_SIZE};
use crate::xhci::{Controller, ProducerRing, Trb, COMPLETION_SUCCESS};
use crate::xhci::{PORT_CONNECTED, PORT_ENABLED, PORT_KEEP, PORT_RESET, PORT_RESET_CHANGE};

const TRB_SETUP: u32 = 2;
const TRB_DATA: u32 = 3;
const TRB_STATUS: u32 = 4;
const TRB_ENABLE_SLOT: u32 = 9;
const TRB_ADDRESS_DEVICE: u32 = 11;
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
    Other,
}

/// A device that was set up (or why not).
#[derive(Debug)]
pub struct Device {
    pub port: u32,
    pub speed: u32,
    pub slot: u32,
    pub vendor: u16,
    pub product: u16,
    pub kind: Kind,
    /// The boot interface's interrupt IN endpoint: (address, packet size, interval).
    pub endpoint: Option<(u8, u16, u8)>,
    pub error: Option<&'static str>,
    /// Kept for reading reports: the device's control ring and buffer page.
    #[allow(dead_code)]
    pub(crate) control: Option<(u64, u64)>,
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

/// Sets up the device on `port`. Never fails as a whole: problems end up
/// in `error`, so the boot log can say what happened on each port.
pub fn enumerate(c: &mut Controller, port: u32) -> Device {
    let mut d = Device {
        port,
        speed: 0,
        slot: 0,
        vendor: 0,
        product: 0,
        kind: Kind::Other,
        endpoint: None,
        error: None,
        control: None,
    };
    if let Err(e) = setup(c, &mut d) {
        d.error = Some(e);
    }
    d
}

fn setup(c: &mut Controller, d: &mut Device) -> Result<(), &'static str> {
    d.speed = reset_port(c, d.port)?;

    let ev =
        c.command(Trb { param: 0, status: 0, control: TRB_ENABLE_SLOT << 10 }).ok_or("no answer to Enable Slot")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("no free device slot");
    }
    let slot = ev.control >> 24;
    d.slot = slot;

    // The device's output context, where the controller keeps its state.
    let output = zeroed_page().ok_or("no memory")?;
    unsafe { (c.dcbaa as *mut u64).add(slot as usize).write_volatile(output) };

    // Input context: control context, slot context, endpoint 0 context.
    let input = zeroed_page().ok_or("no memory")?;
    let mut ring = ProducerRing::new().ok_or("no memory")?;
    let cs = c.ctx_size;
    let packet0: u32 = match d.speed {
        3 => 64,
        4 | 5 => 512,
        _ => 8,
    };
    unsafe {
        let w = |ctx: u64, dword: u64, v: u32| ((input + ctx * cs + 4 * dword) as *mut u32).write_volatile(v);
        w(0, 1, 0b11); // add the slot and endpoint 0
        w(1, 0, d.speed << 20 | 1 << 27); // speed, one context entry
        w(1, 1, d.port << 16); // root hub port
        w(2, 1, 3 << 1 | 4 << 3 | packet0 << 16); // 3 retries, control endpoint, packet size
        w(2, 2, ring.base as u32 | 1); // dequeue pointer, cycle 1
        w(2, 3, (ring.base >> 32) as u32);
        w(2, 4, 8); // average TRB length
    }
    let ev = c
        .command(Trb { param: input, status: 0, control: TRB_ADDRESS_DEVICE << 10 | slot << 24 })
        .ok_or("no answer to Address Device")?;
    if ev.code() != COMPLETION_SUCCESS {
        return Err("the device took no address");
    }

    let buf = zeroed_page().ok_or("no memory")?;
    // Device descriptor.
    let n = control(c, &mut ring, slot, (0x80, 6, 0x0100, 0, 18), buf)?;
    if n < 12 {
        return Err("short device descriptor");
    }
    let desc = unsafe { core::slice::from_raw_parts(buf as *const u8, 18) };
    d.vendor = u16::from_le_bytes([desc[8], desc[9]]);
    d.product = u16::from_le_bytes([desc[10], desc[11]]);

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
    if let (Some(iface), Kind::Keyboard | Kind::Mouse) = (interface, d.kind) {
        // SET_PROTOCOL(boot); SET_IDLE(0) so a keyboard reports only changes.
        control(c, &mut ring, slot, (0x21, 0x0B, 0, iface as u16, 0), buf)?;
        if d.kind == Kind::Keyboard {
            let _ = control(c, &mut ring, slot, (0x21, 0x0A, 0, iface as u16, 0), buf);
        }
    }
    d.control = Some((ring.base, buf));
    Ok(())
}
