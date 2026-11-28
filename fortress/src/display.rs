//! Finds the screen and starts the display service (bin/display).
//!
//! The kernel only finds the screen and hands the service its pieces as
//! capabilities; all drawing is done in user mode.
//!
//! - On a PC (x86_64), when the boot loader (GRUB, through UEFI GOP or
//!   VBE) has already set 800 x 600 at 32 bits, that framebuffer is used
//!   as it is: this is the screen on real PCs. Otherwise the screen is the
//!   standard VGA card under QEMU (the Bochs display, PCI 1234:1111): its
//!   framebuffer is BAR 0 and the service sets the mode itself through the
//!   card's registers (I/O ports 0x1CE-0x1D0). The clock is the CMOS chip
//!   (ports 0x70-0x71).
//! - On AArch64 (QEMU `virt`) the screen is ramfb: the kernel takes a piece
//!   of RAM for the framebuffer and tells the firmware interface (fw_cfg)
//!   where it is, once, at boot. The clock is the PL031 chip.
//!
//! The service's handles: 0 the framebuffer, 1 its control port, 2 the
//! clock, 3 the key focus (where typed keys go), and on a PC 4 the card's
//! mode registers.

use crate::{ipc, mem, process, sched};
use alloc::{string::String, sync::Arc, vec::Vec};

/// The mode the screen runs in: 800 x 600, 32 bits per pixel.
pub const WIDTH: u64 = 800;
pub const HEIGHT: u64 = 600;
/// Pages of framebuffer the service may map.
pub const PAGES: u64 = (WIDTH * HEIGHT * 4).div_ceil(mem::PAGE_SIZE);
/// Where the display service draws Dijana (x, y, width, height), and some
/// of her colours: white, tan, the scarf's red, black.
const DIJANA: (u64, u64, u64, u64) = (600, 550, 64, 48);
const DIJANA_COLOURS: [u32; 4] = [0xF4EFE6, 0xC9803C, 0xCD3228, 0x1E1E26];
/// Tells the display service to stop; it answers with its redraw count
/// (and Dijana's tips times 2^32). With `a0` = `GOOD_NIGHT` Dijana says
/// good night first (the system is shutting down).
pub const QUIT: u64 = 2;
pub const GOOD_NIGHT: u64 = 1;
/// Asks Dijana for a tip after an error; `a0` is the kind (1 to 4).
pub const DIJANA_TIP: u64 = 10;
/// The name programs find the display service by.
pub const NAME: &str = "display";

#[cfg(target_arch = "x86_64")]
mod pc {

    /// The card's mode registers (index and data port).
    pub const DISPI_IO: (u16, u16) = (0x1CE, 3);

    /// The loader's framebuffer, if it is the desk's mode in the pixel
    /// layout the service draws (0x00RRGGBB in a 32-bit word, no padding).
    pub fn boot() -> Option<u64> {
        let fb = crate::pvh::boot_framebuffer()?;
        let fits = fb.kind == 1
            && fb.bpp == 32
            && fb.width as u64 == super::WIDTH
            && fb.height as u64 == super::HEIGHT
            && fb.pitch as u64 == super::WIDTH * 4
            && fb.shifts == (16, 8, 0);
        fits.then_some(fb.addr)
    }

    /// The framebuffer's physical address, if there is such a card.
    pub fn find() -> Option<u64> {
        let f = crate::pci::find(|f| f.id == 0x1111_1234)?;
        // A memory BAR, or it is not the card we know.
        let bar = f.mem_bar(0)?;
        f.enable(0x2);
        crate::pci::map(bar).ok().map(|a| a as u64)
    }
}

#[cfg(target_arch = "aarch64")]
mod ramfb {
    use super::{HEIGHT, PAGES, WIDTH};
    use core::ptr::{read_volatile, write_volatile};
    use core::sync::atomic::{AtomicU64, Ordering};

    /// QEMU `virt`'s firmware configuration interface: data at +0, the
    /// selector at +8 and the DMA address at +16, all big-endian.
    const FW_CFG: usize = 0x0902_0000;
    const FILE_DIR: u16 = 0x19;
    const DMA_ERROR: u32 = 0x01;
    const DMA_SELECT: u32 = 0x08;
    const DMA_WRITE: u32 = 0x10;
    /// 'XR24': 32-bit pixels, 0xXXRRGGBB, the same as on a PC.
    const XRGB8888: u32 = u32::from_le_bytes(*b"XR24");

    /// The framebuffer, once set up (0 = none).
    static FB: AtomicU64 = AtomicU64::new(0);

    #[repr(C)]
    struct DmaAccess {
        control: u32,
        length: u32,
        address: u64,
    }

    #[repr(C, packed)]
    struct Config {
        addr: u64,
        fourcc: u32,
        flags: u32,
        width: u32,
        height: u32,
        stride: u32,
    }

    fn select(key: u16) {
        unsafe { write_volatile((FW_CFG + 8) as *mut u16, key.to_be()) };
    }

    fn read_bytes(buf: &mut [u8]) {
        for b in buf {
            *b = unsafe { read_volatile(FW_CFG as *const u8) };
        }
    }

    /// The selector key of fw_cfg file `name`.
    fn find_file(name: &[u8]) -> Option<u16> {
        select(FILE_DIR);
        let mut n = [0u8; 4];
        read_bytes(&mut n);
        for _ in 0..u32::from_be_bytes(n).min(256) {
            let mut e = [0u8; 64];
            read_bytes(&mut e);
            let end = e[8..].iter().position(|&b| b == 0).unwrap_or(56);
            if &e[8..8 + end] == name {
                return Some(u16::from_be_bytes([e[4], e[5]]));
            }
        }
        None
    }

    /// Takes RAM for the framebuffer and gives it to ramfb, if QEMU has
    /// one (`-device ramfb`). Done once; later calls return the same.
    pub fn find() -> Option<u64> {
        let fb = FB.load(Ordering::Relaxed);
        if fb != 0 {
            return Some(fb);
        }
        let key = find_file(b"etc/ramfb")?;
        let fb = crate::mem::frame::alloc_contiguous(PAGES as usize)?;
        // RAM is identity mapped for the kernel: clear it to black.
        unsafe { core::ptr::write_bytes(fb as *mut u8, 0, (PAGES * crate::mem::PAGE_SIZE) as usize) };
        let cfg = Config {
            addr: fb.to_be(),
            fourcc: XRGB8888.to_be(),
            flags: 0,
            width: (WIDTH as u32).to_be(),
            height: (HEIGHT as u32).to_be(),
            stride: (WIDTH as u32 * 4).to_be(),
        };
        let dma = DmaAccess {
            control: (((key as u32) << 16) | DMA_SELECT | DMA_WRITE).to_be(),
            length: (core::mem::size_of::<Config>() as u32).to_be(),
            address: (&cfg as *const Config as u64).to_be(),
        };
        let dma_ptr = &dma as *const DmaAccess;
        unsafe {
            core::arch::asm!("dsb sy");
            write_volatile((FW_CFG + 16) as *mut u64, (dma_ptr as u64).to_be());
            // QEMU does the transfer during the write; wait anyway.
            let mut spins = 0;
            while u32::from_be(read_volatile(&(*dma_ptr).control)) & !DMA_ERROR != 0 && spins < 1_000_000 {
                spins += 1;
            }
            if u32::from_be(read_volatile(&(*dma_ptr).control)) & DMA_ERROR != 0 {
                for i in 0..PAGES {
                    crate::mem::frame::free(fb + i * crate::mem::PAGE_SIZE);
                }
                return None;
            }
        }
        FB.store(fb, Ordering::Relaxed);
        Some(fb)
    }
}

/// The framebuffer's physical address, if there is a screen.
pub fn find() -> Option<u64> {
    #[cfg(target_arch = "x86_64")]
    return pc::boot().or_else(pc::find);
    #[cfg(target_arch = "aarch64")]
    return ramfb::find();
}

/// A short name for the boot log.
pub fn kind() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    if pc::boot().is_some() {
        return "boot loader's framebuffer (UEFI GOP or VBE)";
    }
    if cfg!(target_arch = "x86_64") {
        "VGA (Bochs display)"
    } else {
        "ramfb"
    }
}

/// The capabilities the display service gets.
fn caps(fb: u64, control: &Arc<ipc::Port>) -> Vec<process::Cap> {
    use process::{Cap, RIGHT_RECV};
    let mut caps = alloc::vec![Cap::Mem { phys: fb, pages: PAGES }, Cap::new(control.clone(), RIGHT_RECV)];
    #[cfg(target_arch = "x86_64")]
    caps.push(Cap::io(0x70, 2));
    #[cfg(target_arch = "aarch64")]
    caps.push(Cap::Mem { phys: 0x0901_0000, pages: 1 }); // the PL031 clock
    caps.push(Cap::KeyFocus);
    // The card's mode registers, unless the loader has set the mode: then
    // the service has no handle 4 and keeps the mode it finds.
    #[cfg(target_arch = "x86_64")]
    if pc::boot() != Some(fb) {
        let (io, len) = pc::DISPI_IO;
        caps.push(Cap::io(io, len));
    }
    caps
}

/// Starts the display service, if there is a screen and the program. It
/// keeps running and keeps the clock going; the returned port stops it.
pub fn start() -> Option<(u64, Arc<ipc::Port>)> {
    let fb = find()?;
    let file = crate::bootfs::image()?.find("bin/display")?;
    let control = ipc::Port::new(16);
    let id = process::spawn_elf("display", file, caps(fb, &control), false).ok()?;
    crate::pointer::set_sink(Some(control.clone()));
    crate::input::set_layout_sink(Some(control.clone()));
    // Programs find it by name to open windows.
    ipc::withdraw(NAME);
    ipc::publish(NAME, control.clone());
    Some((id, control))
}

/// What the display service did while it ran.
pub struct Stopped {
    pub redraws: i64,
    /// Tips Dijana gave after errors.
    pub tips: i64,
}

/// Stops the display service, after Dijana's good night when the system
/// is shutting down (`good_night`).
pub fn stop((id, control): (u64, Arc<ipc::Port>), good_night: bool) -> Stopped {
    crate::console::set_tee(None);
    crate::pointer::set_sink(None);
    crate::input::set_layout_sink(None);
    crate::input::set_focus(None);
    ipc::withdraw(NAME);
    control.send(ipc::Message::new(QUIT, [if good_night { GOOD_NIGHT } else { 0 }, 0, 0]));
    let code = process::wait(id);
    // A program that was stopped answers with a negative code.
    if code < 0 {
        return Stopped { redraws: code, tips: 0 };
    }
    Stopped { redraws: code & 0xFFFF_FFFF, tips: code >> 32 }
}

/// The display service draws the desk from user mode, through a device
/// memory capability, and keeps the clock going; the kernel checks pixels
/// in the framebuffer while it runs, then stops it.
pub fn self_test() -> (bool, String) {
    let Some(fb) = find() else {
        return (true, String::from("skipped: no screen"));
    };
    if crate::bootfs::image().and_then(|fs| fs.find("bin/display")).is_none() {
        return (true, String::from("skipped: no boot image"));
    }
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(service) = start() else {
        return (false, String::from("refused"));
    };
    // Let it run for over two seconds, so the clock ticks at least twice;
    // the pixels are checked while the service is still running.
    sched::sleep(250);
    // The framebuffer is inside the kernel's identity map.
    let px = |x: u64, y: u64| unsafe { ((fb + 4 * (y * WIDTH + x)) as *const u32).read_volatile() } & 0xFF_FFFF;
    let checks = [
        (0, 0, 0x5A8086u32),  // wallpaper, top
        (300, 120, 0xECECEA), // the window's title bar
        (300, 215, 0xFFFFFF), // the window's page
        (404, 500, 0xFFFFFF), // Dijana's greeting bubble
        (600, 290, 0xF4F6F3), // the Clock tile
        (750, 440, 0xF4F6F3), // the Meter tile, under it
        (740, 415, 0xFFFFFF), // the end of the Meter's memory gauge
        (400, 590, 0xE4E8E5), // the Bar
    ];
    let right = checks.iter().filter(|&&(x, y, c)| px(x, y) == c).count();
    let mut ok = right == checks.len();
    // Dijana: her colours in her place, and a frame that changes (her
    // idle animation turns over in under a second).
    let (dx, dy, dw, dh) = DIJANA;
    let snapshot = || {
        let (mut hash, mut hers) = (0u64, 0u32);
        for y in dy..dy + dh {
            for x in dx..dx + dw {
                let c = px(x, y);
                hash = hash.wrapping_mul(31).wrapping_add(c as u64);
                hers += DIJANA_COLOURS.contains(&c) as u32;
            }
        }
        (hash, hers)
    };
    let (first, hers) = snapshot();
    let moved = crate::thread_tests::wait_until(150, || snapshot().0 != first);
    ok &= hers > 300 && moved;
    // Dijana's tip after an error (as Neo DOS asks for one): she answers.
    service.1.send(ipc::Message::new(DIJANA_TIP, [3, 0, 0]));
    let Stopped { redraws, tips } = stop(service, false);
    ok &= redraws >= 2 && tips == 1;
    ok &= crate::thread_tests::wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{}x{} drawn from user mode at {:#x}, {} of {} pixels right, Dijana {} pixels{} ({} tip after an error), clock redrawn {} times, {} frames leaked",
            WIDTH,
            HEIGHT,
            fb,
            right,
            checks.len(),
            hers,
            if moved { " and moving" } else { ", NOT moving" },
            tips,
            redraws,
            leaked
        ),
    )
}

/// The Bar shows the keyboard layout and switches it on a click. A driver
/// switching the layout (as Alt+Shift does) turns the Bar's box to a teal
/// "SR"; a click on the box switches it back to a white "US".
pub fn layout_test() -> (bool, String) {
    let Some(fb) = find() else {
        return (true, String::from("skipped: no screen"));
    };
    if crate::bootfs::image().and_then(|fs| fs.find("bin/display")).is_none() {
        return (true, String::from("skipped: no boot image"));
    }
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    crate::input::set_layout(0);
    let Some(service) = start() else {
        return (false, String::from("refused"));
    };
    let px = || unsafe { ((fb + 4 * (570 * WIDTH + 698)) as *const u32).read_volatile() } & 0xFF_FFFF;
    let (white, teal) = (0xFFFFFF, 0x0D6F78);
    let wait = |c: u32| crate::thread_tests::wait_until(100, || px() == c);
    let mut ok = wait(white);
    let shown_us = ok;

    // A keyboard driver switches the layout.
    crate::input::set_layout(1);
    let shown_sr = wait(teal);
    ok &= shown_sr;

    // Click the box, at (714, 580): move there, press, let go.
    let (x, y, _) = crate::pointer::position();
    crate::pointer::moved(714 - x, 580 - y, 0);
    crate::pointer::moved(0, 0, 1);
    crate::pointer::moved(0, 0, 0);
    let clicked = crate::thread_tests::wait_until(100, || crate::input::layout() == 0) && wait(white);
    ok &= clicked;

    stop(service, false);
    ok &= crate::thread_tests::wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    let say = |b: bool| if b { "" } else { "NOT " };
    (
        ok,
        alloc::format!(
            "Bar {}showed US, {}showed SR after the driver switched, a click {}switched back, {} frames leaked",
            say(shown_us),
            say(shown_sr),
            say(clicked),
            leaked
        ),
    )
}

/// A program opens a window: bin/hello-window finds the display service by
/// name, shares a block of memory for its pixels and passes it in a message.
/// The kernel then finds those pixels on the screen, inside the frame the
/// service drew around them.
pub fn window_test() -> (bool, String) {
    let Some(fb) = find() else {
        return (true, String::from("skipped: no screen"));
    };
    let Some(file) = crate::bootfs::image().and_then(|fs| fs.find("bin/hello-window")) else {
        return (true, String::from("skipped: no boot image"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(service) = start() else {
        return (false, String::from("display service refused"));
    };
    let id = match process::spawn_elf("hello-window", file, alloc::vec![], true) {
        Ok(id) => id,
        Err(e) => return (false, alloc::format!("refused: {}", e)),
    };
    let px = |x: u64, y: u64| unsafe { ((fb + 4 * (y * WIDTH + x)) as *const u32).read_volatile() } & 0xFF_FFFF;
    // The window's content starts below its title bar, at (571, 369).
    let (ox, oy) = (571, 369);
    let checks = [
        (ox + 5, oy + 5, 0x0D6F78u32),  // the teal page
        (ox + 100, oy + 60, 0xFFFFFF),  // the white disc
        (ox + 190, oy + 112, 0xC86F00), // the end of the full progress bar
        (ox + 150, oy - 15, 0xECECEA),  // its title bar, drawn by the service
    ];
    let shown = crate::thread_tests::wait_until(200, || checks.iter().all(|&(x, y, c)| px(x, y) == c));
    let right = checks.iter().filter(|&&(x, y, c)| px(x, y) == c).count();
    let mut ok = shown;
    // Stopping the service tells the program, which then ends.
    stop(service, false);
    let code = process::wait(id);
    ok &= code == 0;
    ok &= crate::thread_tests::wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "hello-window ended with {}, {} of {} pixels of its window right, {} frames leaked",
            code,
            right,
            checks.len(),
            leaked
        ),
    )
}

/// Starts the terminal (bin/terminal): what programs print is copied to
/// it, and what is typed in its window goes to the console. Needs the
/// display service to be running; ends when it stops.
pub fn start_terminal() -> Option<u64> {
    use process::{Cap, RIGHT_RECV};
    let file = crate::bootfs::image()?.find("bin/terminal")?;
    let text = ipc::Port::new(256);
    crate::console::set_tee(Some(text.clone()));
    let id = process::spawn_elf("terminal", file, alloc::vec![Cap::new(text, RIGHT_RECV), Cap::Typing], true).ok();
    if id.is_none() {
        crate::console::set_tee(None);
    }
    id
}

/// The terminal shows console text in its window, and what is typed while
/// its window has the focus reaches the console input.
pub fn terminal_test() -> (bool, String) {
    let Some(fb) = find() else {
        return (true, String::from("skipped: no screen"));
    };
    if crate::bootfs::image().and_then(|fs| fs.find("bin/terminal")).is_none() {
        return (true, String::from("skipped: no boot image"));
    }
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(service) = start() else {
        return (false, String::from("display service refused"));
    };
    let Some(id) = start_terminal() else {
        stop(service, false);
        return (false, String::from("terminal refused"));
    };
    let px = |x: u64, y: u64| unsafe { ((fb + 4 * (y * WIDTH + x)) as *const u32).read_volatile() } & 0xFF_FFFF;
    // The 640 x 400 window fits above the Bar: its text starts at (151, 150).
    let (ox, oy) = (151, 150);
    let page = || crate::thread_tests::wait_until(200, || px(ox + 300, oy + 200) == 0xFFFFFF);
    let mut ok = page();
    // What a program prints shows on the first line (ink pixels).
    let line = b"Neo DOS test\n";
    crate::console::tee(line);
    let ink = || (0..16).any(|y| (0..96).any(|x| px(ox + x, oy + y) == 0x1B2125));
    let shown = crate::thread_tests::wait_until(100, ink);
    ok &= shown;
    // A key typed in its window reaches the console.
    let mut stale = [0u8; 64];
    while crate::input::take(&mut stale) > 0 {}
    crate::input::push(b'z');
    let mut got = [0u8; 4];
    let mut n = 0;
    for _ in 0..100 {
        n += crate::input::take(&mut got[n..]);
        if n > 0 {
            break;
        }
        sched::sleep(1);
    }
    let typed = n == 1 && got[0] == b'z';
    ok &= typed;
    stop(service, false);
    let bytes = process::wait(id);
    ok &= bytes == line.len() as i64;
    ok &= crate::thread_tests::wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "text {} in its window, a key typed there {} the console, it showed {} bytes, {} frames leaked",
            if shown { "shown" } else { "NOT shown" },
            if typed { "reached" } else { "did NOT reach" },
            bytes,
            leaked
        ),
    )
}
