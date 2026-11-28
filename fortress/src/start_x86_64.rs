//! The x86_64 boot path: PVH start info, the kernel's page tables, the
//! scheduler, user programs, and the boot self-tests that use them.

use super::*;
use core::sync::atomic::Ordering;
use program_tests::{bootfs_test, crash_test, elf_test};
use thread_tests::{fp_test, ipc_test, sched_test, sleep_test, wait_until};

/// Called from the architecture entry code with paging on and a stack.
#[no_mangle]
pub extern "C" fn kmain(start_info: usize, magic: u32) -> ! {
    arch::init_console();
    kprintln!("Neo Boing Platinum OS Lite: Fortress {} ({})", VERSION, arch::NAME);
    arch::init_cpu();
    kprintln!("Exception handlers installed.");

    let info = if magic == pvh::MULTIBOOT2_MAGIC {
        kprintln!("Booted by a Multiboot2 loader.");
        unsafe { pvh::BootInfo::from_multiboot2(start_info) }
    } else {
        unsafe { pvh::BootInfo::from_addr(start_info) }
    };
    let Some(info) = info else {
        kprintln!("No PVH or Multiboot2 start info; boot loader not supported.");
        arch::exit_qemu(false);
        arch::halt()
    };

    let mut usable: u64 = 0;
    for e in info.memory_map() {
        kprintln!("  mem {:#012x}..{:#012x} {}", e.addr, e.addr + e.size, e.kind_name());
        if e.kind == pvh::E820_RAM {
            usable += e.size;
        }
    }
    kprintln!("Usable RAM: {} MiB", usable / (1024 * 1024));
    init_frames(&info);

    let mapped = arch::init_memory(ram_end(&info), kernel_image().1);
    mem::frame::FRAMES_ALLOC.lock().set_limit(mapped);
    kprintln!(
        "Kernel page tables active: {} GiB identity mapped, stack guard page at {:#x}",
        mapped >> 30,
        arch::paging::guard_page()
    );

    // From here the log is also drawn on the screen (a black screen on a real PC says nothing otherwise).
    fbcon::init();

    init_bootfs(info.module());

    let (functions, buses) = pci::init();
    kprintln!("PCI: {} functions on {} bus{}.", functions, buses, if buses == 1 { "" } else { "es" });
    for line in pci::known_platform() {
        kprintln!("{}", line);
    }

    match acpi::init(info.rsdp()) {
        Ok(a) => kprintln!(
            "ACPI {}: {} CPU{}, I/O APIC at {:#x}, {} interrupt overrides, power off {}.",
            if a.revision >= 2 { "2.0+" } else { "1.0" },
            a.cpus,
            if a.cpus == 1 { "" } else { "s" },
            a.ioapic,
            a.overrides,
            if a.can_power_off() { "ready" } else { "not available" }
        ),
        Err(e) => kprintln!("ACPI: {}; using PC defaults.", e),
    }

    init_disk();

    match xhci::init() {
        None => kprintln!("No USB controller."),
        Some(Ok(f)) => {
            kprintln!(
                "USB: xHCI {}.{}, {} ports, {} with a device plugged in, {} scratchpad pages; command ring ok; interrupts by {}",
                f.version >> 8,
                (f.version >> 4) & 0xF,
                f.ports,
                f.connected,
                f.scratchpads,
                match f.signal {
                    Some(pci::Signal::Msi) => "MSI",
                    Some(pci::Signal::MsiX) => "MSI-X",
                    None => "polling",
                }
            );
            for d in &f.devices {
                kprintln!("  {}", d.describe());
            }
        }
        Some(Err(e)) => kprintln!("USB controller found but not usable: {}", e),
    }

    if let Some(fb) = display::find() {
        let (w, h, _) = display::mode();
        kprintln!("Screen: {} {}x{} at {:#x}", display::kind(), w, h, fb);
        if (w, h) != display::desk() {
            kprintln!("  The desk ({}x{}) is drawn in the middle of it.", display::desk().0, display::desk().1);
        }
        if let Some(boot) = pvh::boot_framebuffer().filter(|_| !display::kind().starts_with("boot")) {
            kprintln!("  (the boot loader's framebuffer does not fit the desk: {:?})", boot);
        }
    }

    sched::init();
    let timer_hz = arch::init_timer();
    arch::init_input();
    arch::enable_interrupts();
    kprintln!("Timer running at {} Hz (timer clock {} MHz); scheduler started", arch::HZ, timer_hz / 1_000_000);
    // The USB thread reads keyboards and mice, and watches the ports for
    // devices plugged in later.
    if xhci::with(|_| ()).is_some() {
        sched::spawn("usb", usb::thread);
        if usb::reading() {
            kprintln!("USB keyboard and mouse: reading reports.");
        } else {
            kprintln!("USB: no keyboard or mouse yet; watching the ports.");
        }
    }

    let cmdline = info.cmdline();
    if !cmdline.is_empty() {
        kprintln!("Command line: {}", cmdline);
    }
    kprintln!("Clock: {}", clock::describe());
    kprintln!("Fortress is up.");

    for word in cmdline.split(' ') {
        match word {
            "neo.text" => {
                // No desk: the log stays on the screen, to read when the desk shows nothing.
                kprintln!(
                    "Text only: the desk is not started. Read the lines above; the end of the log is the newest."
                );
                arch::halt()
            }
            "neo.test" => self_test(),
            "neo.shell" => run_shell(),
            "neo.test=pagefault" => provoke_page_fault(),
            "neo.test=stackoverflow" => {
                overflow_stack(0);
            }
            _ => {}
        }
    }
    arch::halt()
}

/// Highest address of usable RAM.
fn ram_end(info: &pvh::BootInfo) -> u64 {
    info.memory_map().iter().filter(|e| e.kind == pvh::E820_RAM).map(|e| e.addr + e.size).max().unwrap_or(0)
}

/// Finds room for the frame bitmap: usable RAM above 1 MiB, inside what the
/// boot page tables map, clear of the kernel and the loader's data.
fn place_bitmap(info: &pvh::BootInfo, bytes: u64) -> Option<u64> {
    let (ks, ke) = kernel_image();
    let busy = |a: u64, b: u64| {
        let overlaps = |s: u64, e: u64| s != 0 && a < e && s < b;
        overlaps(ks, ke) || info.occupied().iter().any(|&(s, e)| overlaps(s, e))
    };
    let bytes = bytes.next_multiple_of(mem::PAGE_SIZE);
    for e in info.memory_map().iter().filter(|e| e.kind == pvh::E820_RAM) {
        let end = (e.addr + e.size).min(mem::frame::BOOT_MAPPED);
        let mut at = e.addr.max(0x10_0000).next_multiple_of(mem::PAGE_SIZE);
        while at + bytes <= end {
            if !busy(at, at + bytes) {
                return Some(at);
            }
            at += mem::PAGE_SIZE;
        }
    }
    None
}

/// Hands usable RAM to the frame allocator, minus what is already in use.
fn init_frames(info: &pvh::BootInfo) {
    let end = ram_end(info);
    let bytes = mem::frame::FrameAllocator::bitmap_bytes(end);
    let at = place_bitmap(info, bytes).expect("no room for the frame bitmap");

    let mut fa = mem::frame::FRAMES_ALLOC.lock();
    unsafe { fa.init(at, end) };
    for e in info.memory_map() {
        if e.kind == pvh::E820_RAM {
            fa.add_free(e.addr, e.addr + e.size);
        }
    }
    // The first 1 MiB holds firmware data and the boot page tables of some
    // loaders; keep it out, as most kernels do.
    fa.reserve(0, 0x10_0000);
    let (ks, ke) = kernel_image();
    fa.reserve(ks, ke);
    for (s, e) in info.occupied() {
        if s != 0 {
            fa.reserve(s, e);
        }
    }
    fa.reserve(at, at + bytes);
    let free = fa.free_frames();
    let managed = fa.managed_bytes();
    drop(fa);
    kprintln!("Kernel image {:#x}..{:#x}; frame bitmap {} KiB at {:#x}", ks, ke, bytes.div_ceil(1024), at);
    kprintln!(
        "Free memory {} MiB in {} frames (bitmap covers {} MiB)",
        free as u64 * mem::PAGE_SIZE / (1024 * 1024),
        free,
        managed / (1024 * 1024)
    );
}

/// Boot self-test used by scripts/test-boot-*.sh and CI.
fn self_test() {
    arch::breakpoint();
    let bp = arch::BREAKPOINTS.load(Ordering::Relaxed) == 1;
    kprintln!("Self-test: breakpoint {}", if bp { "caught" } else { "MISSED" });

    let fr = frame_test();
    kprintln!("Self-test: frame allocator {}", if fr { "ok" } else { "FAILED" });

    let pg = paging_test();
    kprintln!("Self-test: page tables {}", if pg { "ok" } else { "FAILED" });

    let hp = heap_test();
    let st = mem::heap::stats();
    kprintln!(
        "Self-test: heap {} ({} KiB mapped, {} KiB free in {} blocks)",
        if hp { "ok" } else { "FAILED" },
        st.mapped / 1024,
        st.free / 1024,
        st.blocks
    );

    let (sc, report) = sched_test();
    kprintln!("Self-test: scheduler {} ({})", if sc { "ok" } else { "FAILED" }, report);

    let (sl, report) = sleep_test();
    kprintln!("Self-test: sleep and park {} ({})", if sl { "ok" } else { "FAILED" }, report);

    let (ip, report) = ipc_test();
    kprintln!("Self-test: message ports {} ({})", if ip { "ok" } else { "FAILED" }, report);

    let (fp, report) = fp_test();
    kprintln!("Self-test: FP registers across switches {} ({})", if fp { "ok" } else { "FAILED" }, report);

    let (us, report) = user_test();
    kprintln!("Self-test: user mode {} ({})", if us { "ok" } else { "FAILED" }, report);

    let (el, report) = elf_test();
    kprintln!("Self-test: Rust programs {} ({})", if el { "ok" } else { "FAILED" }, report);

    let (bf, report) = bootfs_test();
    kprintln!("Self-test: boot image {} ({})", if bf { "ok" } else { "FAILED" }, report);

    let (dk, report) = program_tests::disk_test();
    kprintln!("Self-test: disk {} ({})", if dk { "ok" } else { "FAILED" }, report);

    let (cr, report) = crash_test();
    kprintln!("Self-test: faulty program stopped {} ({})", if cr { "ok" } else { "FAILED" }, report);

    let (co, report) = console_test();
    kprintln!("Self-test: console service {} ({})", if co { "ok" } else { "FAILED" }, report);

    let (kb, report) = keyboard_test();
    kprintln!("Self-test: keyboard service {} ({})", if kb { "ok" } else { "FAILED" }, report);

    let (dp, report) = display::self_test();
    kprintln!("Self-test: display service {} ({})", if dp { "ok" } else { "FAILED" }, report);

    let (pt, report) = pointer_test();
    kprintln!("Self-test: mouse and pointer {} ({})", if pt { "ok" } else { "FAILED" }, report);

    let (wn, report) = display::window_test();
    kprintln!("Self-test: a program's window {} ({})", if wn { "ok" } else { "FAILED" }, report);

    let (ck, report) = click_test();
    kprintln!("Self-test: clicks and keys in a window {} ({})", if ck { "ok" } else { "FAILED" }, report);

    let (tm, report) = display::terminal_test();
    kprintln!("Self-test: terminal window {} ({})", if tm { "ok" } else { "FAILED" }, report);

    let (rg, report) = crate::ring_tests::ring_test();
    kprintln!("Self-test: shared-memory ring {} ({})", if rg { "ok" } else { "FAILED" }, report);

    let (ly, report) = display::layout_test();
    kprintln!("Self-test: layout switch on the Bar {} ({})", if ly { "ok" } else { "FAILED" }, report);

    let (ca, report) = calculator_test();
    kprintln!("Self-test: Calculator tile {} ({})", if ca { "ok" } else { "FAILED" }, report);

    arch::exit_qemu(
        bp && fr
            && pg
            && hp
            && sc
            && sl
            && ip
            && fp
            && us
            && el
            && bf
            && dk
            && cr
            && co
            && kb
            && dp
            && pt
            && wn
            && ck
            && tm
            && ly
            && ca
            && rg,
    );
}

/// Presses a key as if on the keyboard: the PS/2 controller's command 0xD2
/// puts a byte in its output buffer and raises IRQ 1, so the test needs no
/// one at the keyboard. Waits until the driver has read it.
fn inject_scancode(code: u8) -> bool {
    let input_empty = || unsafe { arch::inb(0x64) } & 2 == 0;
    let ok = wait_until(10, input_empty);
    unsafe { arch::outb(0x64, 0xD2) };
    let ok = ok && wait_until(10, input_empty);
    unsafe { arch::outb(0x60, code) };
    ok && wait_until(50, || unsafe { arch::inb(0x64) } & 1 == 0)
}

/// What the PS/2 controller's driver gets for keyboard and mouse: its
/// ports, both interrupt lines, typing and pointer capabilities.
pub fn ps2_caps() -> alloc::vec::Vec<process::Cap> {
    use process::Cap;
    irq::claim(1);
    irq::claim(12);
    alloc::vec![Cap::io(0x60, 5), Cap::Irq(1), Cap::Typing, Cap::Irq(12), Cap::Pointer]
}

/// Moves the mouse as if by hand: the controller's command 0xD3 puts a byte
/// in its output buffer as if the mouse sent it, and raises IRQ 12.
fn inject_mouse(byte: u8) -> bool {
    let input_empty = || unsafe { arch::inb(0x64) } & 2 == 0;
    let ok = wait_until(10, input_empty);
    unsafe { arch::outb(0x64, 0xD3) };
    let ok = ok && wait_until(10, input_empty);
    unsafe { arch::outb(0x60, byte) };
    ok && wait_until(50, || unsafe { arch::inb(0x64) } & 1 == 0)
}

/// The mouse: the PS/2 driver turns packets into pointer moves, the kernel
/// keeps the position and the display service draws the arrow there.
fn pointer_test() -> (bool, alloc::string::String) {
    let Some(file) = bootfs::image().and_then(|fs| fs.find("bin/keyboard")) else {
        return (true, alloc::string::String::from("skipped: no boot image"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(fb) = display::find() else {
        return (true, alloc::string::String::from("skipped: no screen"));
    };
    let Some(screen) = display::start() else {
        return (false, alloc::string::String::from("display service refused"));
    };
    let id = match process::spawn_elf("keyboard", file, ps2_caps(), false) {
        Ok(id) => id,
        Err(e) => return (false, alloc::format!("refused: {}", e)),
    };
    sched::sleep(20);
    let moves = pointer::MOVES.load(Ordering::Relaxed);
    let (x0, y0, _) = pointer::position();
    // Right 40; down 30 (the mouse counts y upwards: -30, sign bit 5);
    // then the left button down.
    let packets = [[0x08, 40, 0], [0x28, 0, 0xE2], [0x09, 0, 0]];
    let mut ok = packets.iter().flatten().all(|&b| inject_mouse(b));
    ok &= wait_until(50, || pointer::MOVES.load(Ordering::Relaxed) - moves >= 3);
    let (x, y, buttons) = pointer::position();
    ok &= (x, y, buttons) == (x0 + 40, y0 + 30, 1);
    // The arrow's tip (ink) and a pixel inside it (white) at the new place.
    let px = |x: i64, y: i64| display::pixel(display::view(fb), x as u64, y as u64);
    let drawn = wait_until(50, || px(x, y) == 0 && px(x + 1, y + 5) == 0xFFFFFF);
    ok &= drawn;
    // Button up again, so the shell starts with it released.
    ok &= [0x08, 0, 0].iter().all(|&b| inject_mouse(b));
    ok &= wait_until(50, || pointer::position().2 == 0);

    // Drag the window by its title bar: to (300, 110) (left 140: sign bit
    // 4; up 220, which the mouse counts as +220), press, move right 100
    // and down 150 (-150: sign bit 5), release. The window (at 90, 100)
    // ends up at (190, 250).
    let drag = [[0x18, 116, 220], [0x09, 0, 0], [0x29, 100, 106], [0x08, 0, 0]];
    ok &= drag.iter().flatten().all(|&b| inject_mouse(b));
    ok &= wait_until(50, || pointer::position() == (400, 260, 0));
    let window = 0xECECEA;
    // The title bar at its new place; the old place shows the wallpaper
    // again (the same colour across a row).
    let dragged = wait_until(50, || px(250, 260) == window && px(120, 110) == px(5, 110));
    ok &= dragged;
    display::stop(screen, false);
    irq::revoke(1);
    irq::revoke(12);
    let typed = process::wait(id);
    ok &= typed == 0;
    ok &= wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{} packets through IRQ 12, pointer at ({}, {}) with button {}, arrow {}, window {}, {} frames leaked",
            packets.len(),
            x,
            y,
            buttons,
            if drawn { "drawn" } else { "MISSING" },
            if dragged { "dragged" } else { "NOT MOVED" },
            leaked
        ),
    )
}

/// Moves the injected mouse to `(x, y)` from where the pointer is, then
/// sets the buttons; each packet moves at most 127 pixels per axis.
fn mouse_to(x: i64, y: i64, buttons: u8) -> bool {
    let mut ok = true;
    loop {
        // Moving keeps the buttons as they are; they change at the end.
        let (px, py, held) = pointer::position();
        let (dx, dy) = ((x - px).clamp(-127, 127), (y - py).clamp(-127, 127));
        if dx == 0 && dy == 0 {
            break;
        }
        // The mouse counts y upwards; sign bits 4 (x) and 5 (y).
        let my = -dy;
        let flags = 0x08 | if dx < 0 { 0x10 } else { 0 } | if my < 0 { 0x20 } else { 0 };
        let moves = pointer::MOVES.load(Ordering::Relaxed);
        ok &= [flags | held as u8, dx as u8, my as u8].iter().all(|&b| inject_mouse(b));
        ok &= wait_until(50, || pointer::MOVES.load(Ordering::Relaxed) > moves);
        if !ok {
            return false;
        }
    }
    let moves = pointer::MOVES.load(Ordering::Relaxed);
    ok &= [0x08 | (buttons & 7), 0, 0].iter().all(|&b| inject_mouse(b));
    ok && (pointer::position().2 == buttons as u64 || wait_until(50, || pointer::MOVES.load(Ordering::Relaxed) > moves))
}

/// The Calculator tile, clicked with the injected mouse: 7 * 6 = shows the
/// same pixels as typing 42 after C, and 1 / 0 = makes Dijana answer (a
/// tip) instead of a crash.
fn calculator_test() -> (bool, alloc::string::String) {
    let Some(kb) = bootfs::image().and_then(|f| f.find("bin/keyboard")) else {
        return (true, alloc::string::String::from("skipped: no boot image"));
    };
    let Some(fb) = display::find() else {
        return (true, alloc::string::String::from("skipped: no screen"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(screen) = display::start() else {
        return (false, alloc::string::String::from("display service refused"));
    };
    let Ok(driver) = process::spawn_elf("keyboard", kb, ps2_caps(), false) else {
        return (false, alloc::string::String::from("refused"));
    };
    sched::sleep(20);
    // The keys as the display service lays them out (bottom left).
    const KEYS: [&[u8; 4]; 4] = [b"789/", b"456*", b"123-", b"C0=+"];
    let press = |k: u8| {
        let Some((row, col)) =
            KEYS.iter().enumerate().find_map(|(r, keys)| keys.iter().position(|&c| c == k).map(|c| (r, c)))
        else {
            return false;
        };
        let by = display::desk().1 as i64 - display::HEIGHT as i64;
        let (x, y) = (16 + col as i64 * 32 + 14, 438 + by + row as i64 * 28 + 12);
        let ok = mouse_to(x, y, 1) && mouse_to(x, y, 0);
        sched::sleep(5);
        ok
    };
    // The calculator's display.
    let shown = || {
        let mut hash = 0u64;
        let by = display::desk().1 - display::HEIGHT;
        for y in 408 + by..430 + by {
            for x in 17..139u64 {
                let c = display::pixel(display::view(fb), x, y);
                hash = hash.wrapping_mul(31).wrapping_add(c as u64);
            }
        }
        hash
    };
    let mut ok = true;
    let zero = shown();
    ok &= b"7*6=".iter().all(|&k| press(k));
    sched::sleep(10);
    let result = shown();
    ok &= b"C".iter().all(|&k| press(k));
    sched::sleep(10);
    let cleared = shown() == zero;
    ok &= b"42".iter().all(|&k| press(k));
    sched::sleep(10);
    let right = shown() == result && result != zero;
    ok &= cleared && right;
    ok &= b"1/0=".iter().all(|&k| press(k));
    sched::sleep(10);
    let stopped = display::stop(screen, false);
    ok &= stopped.tips == 1;
    irq::revoke(1);
    irq::revoke(12);
    ok &= process::wait(driver) == 0;
    ok &= wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "7*6= {}, C {}, 1/0= gave {} tip, {} frames leaked",
            if right { "shows 42" } else { "WRONG" },
            if cleared { "clears" } else { "does NOT clear" },
            stopped.tips,
            leaked
        ),
    )
}

/// Clicks and keys reach a program's window: a click on hello-window's disc
/// turns it amber (the program redraws) and gives the window the keys;
/// typed letters appear in it, and Esc makes the program close its window
/// and end. Then keys go to the console again.
fn click_test() -> (bool, alloc::string::String) {
    let find = |name| bootfs::image().and_then(|f| f.find(name));
    let (Some(kb), Some(hw)) = (find("bin/keyboard"), find("bin/hello-window")) else {
        return (true, alloc::string::String::from("skipped: no boot image"));
    };
    let Some(fb) = display::find() else {
        return (true, alloc::string::String::from("skipped: no screen"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let Some(screen) = display::start() else {
        return (false, alloc::string::String::from("display service refused"));
    };
    let (Ok(driver), Ok(program)) = (
        process::spawn_elf("keyboard", kb, ps2_caps(), false),
        process::spawn_elf("hello-window", hw, alloc::vec![], true),
    ) else {
        return (false, alloc::string::String::from("refused"));
    };
    let px = |x: i64, y: i64| display::pixel(display::view(fb), x as u64, y as u64);
    // The disc's middle, a pixel inside it away from the arrow, and the
    // window's title bar.
    let (cx, cy) = (571 + 100, 369 + 60);
    let (dx, dy) = (cx - 20, cy + 10);
    // Let the driver set the controller up, and the window appear.
    sched::sleep(20);
    let mut ok = wait_until(200, || px(dx, dy) == 0xFFFFFF);
    ok &= mouse_to(cx, cy, 1) && mouse_to(cx, cy, 0);
    let turned = wait_until(100, || px(dx, dy) == 0xC86F00);
    ok &= turned;
    // The window has the keys now: "hi" appears on its line (white
    // pixels where it was all teal), Esc closes it. None of it reaches the
    // console.
    let mut stale = [0u8; 64];
    while input::take(&mut stale) > 0 {}
    ok &= [0x23, 0xA3, 0x17, 0x97].iter().all(|&c| inject_scancode(c));
    let line = |x: i64, y: i64| (0..16).any(|dy| (0..16).any(|dx| px(x + dx, y + dy) == 0xFFFFFF));
    let typed = wait_until(100, || line(571 + 8, 369 + 86));
    ok &= typed;
    // The Bar lists two windows (the second button's left edge), and
    // hello-window's button is lit (the front window).
    let bar_y = display::desk().1 as i64 - 20;
    let listed = px(184, bar_y) == 0x9AA19C && px(190, bar_y - 8) == 0x0D6F78;
    ok &= listed;
    // Its minimize box (a bar in a square) minimizes it: the desk shows again where it was, and
    // its button turns grey. A click on the button brings it back, the
    // disc still amber.
    ok &= mouse_to(570 + 162, 340 + 14, 1) && mouse_to(570 + 162, 340 + 14, 0);
    let mut minimized = wait_until(100, || px(721, 470) == px(5, 470) && px(190, bar_y - 8) == 0xECECEA);
    ok &= mouse_to(230, bar_y, 1) && mouse_to(230, bar_y, 0);
    minimized &= wait_until(100, || px(dx, dy) == 0xC86F00 && px(190, bar_y - 8) == 0x0D6F78);
    ok &= minimized;
    // A click on its close box (the X) asks it to close; it ends
    // as on Esc, with ten times its colour changes plus the typed line.
    ok &= mouse_to(570 + 186, 340 + 14, 1) && mouse_to(570 + 186, 340 + 14, 0);
    let result = process::wait(program);
    ok &= result == 12;
    // A point the window covered, below the tiles: the wallpaper again;
    // and its button has left the Bar.
    let closed = wait_until(100, || px(721, 470) == px(5, 470) && px(184, bar_y) == 0xE4E8E5);
    ok &= closed;
    ok &= input::take(&mut stale) == 0;
    // With the window gone, keys go to the console again.
    ok &= [0x2D, 0xAD].iter().all(|&c| inject_scancode(c));
    let mut got = [0u8; 4];
    let mut n = 0;
    for _ in 0..50 {
        n += input::take(&mut got[n..]);
        if n > 0 {
            break;
        }
        sched::sleep(1);
    }
    let back = n == 1 && got[0] == b'x';
    ok &= back;
    // The Welcome window's close box hides it (white page gone); About in
    // the Neo menu brings it back.
    ok &= mouse_to(90 + 444, 100 + 14, 1) && mouse_to(90 + 444, 100 + 14, 0);
    let hidden = wait_until(100, || px(500, 140) != 0xFFFFFF && px(81, bar_y) == 0xE4E8E5);
    ok &= mouse_to(30, bar_y, 1) && mouse_to(30, bar_y, 0);
    ok &= mouse_to(60, display::desk().1 as i64 - 41 - 4 - 86 + 16, 1);
    ok &= mouse_to(60, display::desk().1 as i64 - 41 - 4 - 86 + 16, 0);
    let again = hidden && wait_until(100, || px(500, 140) == 0xFFFFFF);
    ok &= again;
    display::stop(screen, false);
    irq::revoke(1);
    irq::revoke(12);
    // The driver typed three keys: H, I and X.
    ok &= process::wait(driver) == 3;
    ok &= wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "click {} the disc, typing {} in the window, the Bar {} it, it {} and came back, its close box {} it, program ended with {}, keys {} the console after, Welcome {}, {} frames leaked",
            if turned { "turned" } else { "did NOT turn" },
            if typed { "showed" } else { "did NOT show" },
            if listed { "listed" } else { "did NOT list" },
            if minimized { "minimized" } else { "did NOT minimize" },
            if closed { "closed" } else { "did NOT close" },
            result,
            if back { "reach" } else { "do NOT reach" },
            if again { "closed and came back" } else { "did NOT close and come back" },
            leaked
        ),
    )
}

/// The keyboard service runs the PS/2 controller from user mode: an I/O
/// capability for its ports, an interrupt capability for IRQ 1 and a
/// typing capability. What it types reaches the console input.
fn keyboard_test() -> (bool, alloc::string::String) {
    use process::Cap;
    let Some(file) = bootfs::image().and_then(|fs| fs.find("bin/keyboard")) else {
        return (true, alloc::string::String::from("skipped: no boot image"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let mut stale = [0u8; 64];
    while input::take(&mut stale) > 0 {}

    irq::claim(1);
    let caps = alloc::vec![Cap::io(0x60, 5), Cap::Irq(1), Cap::Typing];
    let id = match process::spawn_elf("keyboard", file, caps, false) {
        Ok(id) => id,
        Err(e) => return (false, alloc::format!("refused: {}", e)),
    };
    // Let it set the controller up and wait on its interrupt.
    sched::sleep(10);

    // Shift+H, I, space, 4, 2, Enter (scan code set 1; 0x80 = release).
    // Then Alt+Shift switches to Serbian Latin: the key right of L gives č
    // (two bytes of UTF-8) and Y's place gives z. Alt+Shift switches back.
    let codes = [
        0x2A, 0x23, 0xA3, 0xAA, 0x17, 0x97, 0x39, 0xB9, 0x05, 0x85, 0x03, 0x83, 0x1C, 0x9C, // Hi 42
        0x38, 0x2A, 0xAA, 0xB8, 0x27, 0xA7, 0x15, 0x95, 0x38, 0x2A, 0xAA, 0xB8, 0x1C, 0x9C, // čz
    ];
    let mut ok = codes.iter().all(|&c| inject_scancode(c));
    let mut got = [0u8; 16];
    let mut n = 0;
    for _ in 0..100 {
        n += input::take(&mut got[n..]);
        if n >= 10 {
            break;
        }
        sched::sleep(1);
    }
    let typed = &got[..n];
    ok &= typed == "Hi 42\nčz\n".as_bytes();

    // Unplug it: the driver's wait fails and it ends with its count.
    irq::revoke(1);
    let count = process::wait(id);
    ok &= count == 10;

    ok &= wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{} keys typed {:?} through IRQ 1, driver ended with {}, {} frames leaked",
            codes.len(),
            core::str::from_utf8(typed).unwrap_or("?"),
            count,
            leaked
        ),
    )
}

/// The console service owns the serial port (an I/O capability); a client
/// prints through it with nothing but a send right.
fn console_test() -> (bool, alloc::string::String) {
    use process::{Cap, LAST_EXIT, RIGHT_RECV, RIGHT_SEND};
    let mut ok = true;
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;

    let port = ipc::Port::new(8);
    let (io_base, io_len) = arch::CONSOLE_IO;
    let console = process::spawn_elf(
        "console",
        programs::console_elf(),
        alloc::vec![Cap::new(port.clone(), RIGHT_RECV), Cap::io(io_base, io_len)],
        false,
    );
    ok &= console.is_ok();

    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);
    let client = process::spawn_elf(
        "hello-console",
        programs::hello_console_elf(),
        alloc::vec![Cap::new(port.clone(), RIGHT_SEND)],
        false,
    );
    ok &= client.is_ok();
    ok &= wait_until(200, || LAST_EXIT.load(Ordering::Relaxed) == 0);
    let client_exit = LAST_EXIT.load(Ordering::Relaxed);

    // Ask the service to stop; it exits with the number of characters printed.
    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);
    port.send(ipc::Message::new(CONSOLE_QUIT, [0, 0, 0]));
    ok &= wait_until(200, || LAST_EXIT.load(Ordering::Relaxed) != i64::MIN);
    let chars = LAST_EXIT.load(Ordering::Relaxed);
    ok &= chars > 0;
    drop(port);

    ok &= wait_until(100, || sched::thread_count() == threads);
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;

    (ok, alloc::format!("client exit {}, {} characters printed, {} frames leaked", client_exit, chars, leaked))
}

/// The console protocol's stop request (see user/neo-rt).
const CONSOLE_QUIT: u64 = 2;

fn user_test() -> (bool, alloc::string::String) {
    use process::{LAST_EXIT, STOPPED};
    let mut ok = true;
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;

    // A well-behaved program: prints, finds a port by name, sends a
    // message, sleeps and exits with 0.
    let port = ipc::Port::new(4);
    ipc::publish("neo.test.user", port.clone());
    let id = process::spawn("hello", programs::hello(), alloc::vec![], true);
    let got = wait_until(100, || port.try_recv().is_some_and(|m| m.tag == 0x100 && m.args == [42, 7, id]));
    ok &= got;
    ok &= wait_until(100, || LAST_EXIT.load(Ordering::Relaxed) == 0);
    ipc::withdraw("neo.test.user");

    // A bad pointer is refused with an error code, not a crash.
    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);
    process::spawn("badptr", programs::badptr(), alloc::vec![], false);
    ok &= wait_until(100, || LAST_EXIT.load(Ordering::Relaxed) == syscall::ERR_BAD_POINTER as i64);

    // Programs that touch kernel memory or run privileged instructions are
    // stopped; the kernel carries on.
    let stopped = STOPPED.load(Ordering::Relaxed);
    process::spawn("peek", programs::peek(), alloc::vec![], false);
    process::spawn("halt", programs::halt(), alloc::vec![], false);
    ok &= wait_until(100, || STOPPED.load(Ordering::Relaxed) == stopped + 2);

    // Capabilities: a service in user mode and a client that holds only
    // the send right to it. The client checks it is refused everything it
    // was not given, then calls the service and exits with the answer.
    let service = ipc::Port::new(4);
    use process::{Cap, RIGHT_RECV, RIGHT_SEND};
    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);
    process::spawn("doubler", programs::doubler(), alloc::vec![Cap::new(service.clone(), RIGHT_RECV)], false);
    process::spawn("client", programs::client(), alloc::vec![Cap::new(service.clone(), RIGHT_SEND)], false);
    let client_ok = wait_until(100, || LAST_EXIT.load(Ordering::Relaxed) == 42);
    if !client_ok {
        kprintln!("  client exit code: {}", LAST_EXIT.load(Ordering::Relaxed));
    }
    ok &= client_ok;
    // The kernel holds the port too and can call the service directly.
    ok &= service.call(ipc::Message::new(1, [500, 0, 0])).args[0] == 1000;
    service.send(ipc::Message::new(2, [0; 3]));
    ok &= wait_until(100, || LAST_EXIT.load(Ordering::Relaxed) == 0);
    drop(service);

    // Everything is cleaned up: threads, address spaces, stacks.
    ok &= wait_until(100, || sched::thread_count() == threads);
    // Frames the heap took meanwhile stay with the heap; all others must
    // have come back.
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    let leaked = free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64;
    ok &= leaked == 0;

    (ok, alloc::format!("6 programs, {} stopped, {} frames leaked", STOPPED.load(Ordering::Relaxed) - stopped, leaked))
}

fn paging_test() -> bool {
    let space = arch::paging::kernel_space();
    let guard = arch::paging::guard_page();
    let (ks, _) = kernel_image();
    let kmain_addr = kmain as *const () as u64;
    space.translate(guard).is_none()
        && space.translate(guard + 4096) == Some(guard + 4096)
        && space.translate(ks) == Some(ks)
        && space.translate(kmain_addr) == Some(kmain_addr)
        && space.translate(0xFEE0_0000) == Some(0xFEE0_0000) // local APIC
}

/// Reads memory that is not mapped, to check that the fault is reported.
fn provoke_page_fault() {
    // The boot page tables map only the first 1 GiB.
    let p = 0x40_0000_0000usize as *const u64;
    let v = unsafe { core::ptr::read_volatile(p) };
    kprintln!("Read {:#x} from unmapped memory; this should not happen.", v);
}
