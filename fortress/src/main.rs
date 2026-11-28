//! Fortress, the Neo Boing Platinum OS microkernel. Bootstrap stage.
//!
//! Neo Boing Platinum OS Lite is built in small steps (see docs/PROGRESS.md). On x86_64 the
//! kernel runs a scheduler and isolated user programs (start_x86_64.rs); the
//! AArch64 port is younger and follows step by step (start_aarch64.rs).

#![no_std]
#![no_main]

extern crate alloc;

#[cfg(target_arch = "x86_64")]
mod acpi;
mod arch;
mod bootfs;
mod clock;
mod console;
mod diskfs;
mod display;
mod fbcon;
mod mem;
#[cfg(target_arch = "x86_64")]
mod nvme;
#[cfg(target_arch = "x86_64")]
mod pci;
mod sync;

mod elf;
mod input;
mod ipc;
mod irq;
mod pointer;
mod process;
mod program_tests;
mod programs;
#[cfg(target_arch = "x86_64")]
mod pvh;
mod ring_tests;
mod sched;
#[cfg(target_arch = "x86_64")]
mod start_x86_64;
mod syscall;
mod thread_tests;
#[cfg(target_arch = "x86_64")]
mod usb;
mod virtio_blk;
#[cfg(target_arch = "aarch64")]
mod virtio_input;
#[cfg(target_arch = "x86_64")]
mod xhci;

#[cfg(target_arch = "aarch64")]
mod fdt;
#[cfg(target_arch = "aarch64")]
mod start_aarch64;

use core::panic::PanicInfo;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

fn kernel_image() -> (u64, u64) {
    (core::ptr::addr_of!(__kernel_start) as u64, core::ptr::addr_of!(__kernel_end) as u64)
}

/// Frame allocator self-test, the same on every architecture.
fn frame_test() -> bool {
    use mem::{frame, PAGE_SIZE};
    let before = frame::free_frames();
    let (ks, ke) = kernel_image();
    let (Some(a), Some(b), Some(c)) = (frame::alloc(), frame::alloc(), frame::alloc()) else {
        return false;
    };
    let fresh = |x: u64| x % PAGE_SIZE == 0 && x >= 0x10_0000 && !(ks..ke).contains(&x);
    let mut ok = a != b && b != c && a != c && fresh(a) && fresh(b) && fresh(c);
    ok &= frame::free_frames() == before - 3;
    // The frame must be real, writable RAM.
    unsafe {
        let p = b as *mut u64;
        p.write_volatile(0x4E45_4F21);
        ok &= p.read_volatile() == 0x4E45_4F21;
    }
    // A freed frame is the next one handed out (lowest first).
    frame::free(b);
    ok &= frame::alloc() == Some(b);
    for x in [a, b, c] {
        frame::free(x);
    }
    ok && frame::free_frames() == before
}

fn heap_test() -> bool {
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::alloc::Layout;
    let before = mem::heap::stats();

    // A growing vector: forces reallocation and heap growth.
    let mut v: Vec<u64> = Vec::new();
    for i in 0..100_000u64 {
        v.push(i);
    }
    let mut ok = v.iter().sum::<u64>() == 100_000 * 99_999 / 2;
    drop(v);

    // Many small boxes; free every other one, refill the holes, check contents.
    let mut boxes: Vec<Option<Box<[u8; 100]>>> = (0..1000).map(|i| Some(Box::new([i as u8; 100]))).collect();
    for b in boxes.iter_mut().step_by(2) {
        *b = None;
    }
    for (i, b) in boxes.iter_mut().enumerate().step_by(2) {
        *b = Some(Box::new([(i as u8).wrapping_add(7); 100]));
    }
    for (i, b) in boxes.iter().enumerate() {
        let want = if i % 2 == 0 { (i as u8).wrapping_add(7) } else { i as u8 };
        ok &= b.as_ref().is_some_and(|b| b.iter().all(|&x| x == want));
    }
    drop(boxes);

    // Page-aligned allocation.
    unsafe {
        let layout = Layout::from_size_align(5000, 4096).unwrap();
        let p = alloc::alloc::alloc(layout);
        ok &= !p.is_null() && p as usize % 4096 == 0;
        if !p.is_null() {
            p.write_bytes(0xAB, 5000);
            alloc::alloc::dealloc(p, layout);
        }
    }

    let mut s = String::from("Neo");
    s.push_str(" Lite");
    ok &= s == "Neo Lite";
    drop(s);

    // Everything the test took was given back, and merged again: the free
    // space did not split into more pieces than before.
    let st = mem::heap::stats();
    ok && st.mapped > 0 && st.free - before.free == st.mapped - before.mapped && st.blocks <= before.blocks.max(1)
}

/// Recurses until the stack runs into the guard page.
#[allow(unconditional_recursion)]
#[inline(never)]
fn overflow_stack(depth: u64) -> u64 {
    let mut pad = [depth; 64];
    let pad = core::hint::black_box(&mut pad);
    pad[0] + overflow_stack(depth + 1) + pad[63]
}

/// Takes over the boot image, if the loader gave one, and reports it.
fn init_bootfs(range: Option<(u64, u64)>) {
    let Some((start, end)) = range else {
        kprintln!("No boot image.");
        return;
    };
    match unsafe { bootfs::init(start, end) } {
        Ok(fs) => kprintln!("Boot image at {:#x}: {} files, {} KiB", start, fs.files().count(), fs.size() / 1024),
        Err(e) => kprintln!("Boot image at {:#x} refused: {}", start, e),
    }
}

/// Looks for a disk and reports it: a virtio disk, or on a PC an NVMe disk
/// when there is no virtio one.
fn init_disk() {
    match virtio_blk::init() {
        Some(Ok(sectors)) => {
            kprintln!("Disk: virtio, {} sectors ({} KiB)", sectors, sectors * virtio_blk::SECTOR as u64 / 1024)
        }
        Some(Err(e)) => kprintln!("Disk found but not usable: {}", e),
        None => {
            #[cfg(target_arch = "x86_64")]
            match nvme::init() {
                Some(Ok(sectors)) => {
                    kprintln!(
                        "Disk: NVMe, {} sectors ({} MiB){}",
                        sectors,
                        sectors * nvme::SECTOR as u64 / (1024 * 1024),
                        if nvme::writable() { "" } else { ", read only (not a Neo disk)" }
                    );
                    return;
                }
                Some(Err(e)) => {
                    kprintln!("NVMe disk found but not usable: {}", e);
                    return;
                }
                None => {}
            }
            kprintln!("No disk.");
        }
    }
}

/// Starts Neo DOS from the boot image with its handles: 0 the boot image,
/// 1 the console, 2 the disk (when there is one). Waits for it to end and
/// leaves QEMU: exit status 33 when it ended normally.
fn run_shell() {
    let Some(file) = bootfs::image().and_then(|fs| fs.find("bin/neodos")) else {
        kprintln!("No bin/neodos in the boot image.");
        arch::exit_qemu(false);
        return;
    };
    use process::{Cap, Volume};
    let mut caps = alloc::vec![Cap::Dir(Volume::Boot), Cap::Console];
    if virtio_blk::sectors().is_some() {
        caps.push(Cap::Dir(Volume::Disk));
    }
    // The display service draws the desk and keeps running beside Neo DOS;
    // the kernel holds its control port and stops it when Neo DOS ends.
    let screen = display::start();
    // Neo DOS also shows in a window on the desk.
    let terminal = screen.as_ref().and_then(|_| display::start_terminal());
    // On a PC the keyboard service lets Neo DOS read the PS/2 keyboard too,
    // and moves the pointer with the PS/2 mouse.
    #[cfg(target_arch = "x86_64")]
    if let Some(kb) = bootfs::image().and_then(|fs| fs.find("bin/keyboard")) {
        if let Err(e) = process::spawn_elf("keyboard", kb, start_x86_64::ps2_caps(), false) {
            kprintln!("Keyboard service refused: {}", e);
        }
    }
    // Neo DOS may look ports up by name, so the programs it starts can
    // find the display service and open windows.
    match process::spawn_elf("neodos", file, caps, true) {
        Ok(id) => {
            let code = process::wait(id);
            kprintln!(
                "Neo DOS ended (code {}), {} input interrupts.",
                code,
                input::IRQS.load(core::sync::atomic::Ordering::Relaxed)
            );
            let (x, y, _) = pointer::position();
            kprintln!(
                "Pointer: {} moves, at ({}, {}).",
                pointer::MOVES.load(core::sync::atomic::Ordering::Relaxed),
                x,
                y
            );
            #[cfg(target_arch = "x86_64")]
            if usb::PLUGGED.load(core::sync::atomic::Ordering::Relaxed) > 0 {
                kprintln!(
                    "USB hot-plug: {} plugged in, {} pulled out.",
                    usb::PLUGGED.load(core::sync::atomic::Ordering::Relaxed),
                    usb::UNPLUGGED.load(core::sync::atomic::Ordering::Relaxed)
                );
            }
            #[cfg(target_arch = "x86_64")]
            if usb::reading() || usb::PLUGGED.load(core::sync::atomic::Ordering::Relaxed) > 0 {
                kprintln!(
                    "USB: {} keys, {} mouse reports, {} interrupts.",
                    usb::KEYS.load(core::sync::atomic::Ordering::Relaxed),
                    usb::MOUSE_REPORTS.load(core::sync::atomic::Ordering::Relaxed),
                    usb::INTERRUPTS.load(core::sync::atomic::Ordering::Relaxed)
                );
            }
            if let Some(service) = screen {
                let s = display::stop(service, true);
                kprintln!(
                    "Display service stopped after {} clock redraws; Dijana gave {} tip{} and said good night.",
                    s.redraws,
                    s.tips,
                    if s.tips == 1 { "" } else { "s" }
                );
            }
            if let Some(id) = terminal {
                kprintln!("Terminal showed {} bytes.", process::wait(id));
            }
            arch::exit_qemu(code == 0);
            // Not in QEMU's test setup: switch a real PC off.
            #[cfg(target_arch = "x86_64")]
            {
                kprintln!("Switching off.");
                acpi::power_off();
            }
        }
        Err(e) => {
            kprintln!("Neo DOS refused: {}", e);
            arch::exit_qemu(false);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    fbcon::resume();
    kprintln!("FORTRESS PANIC: {}", info.message());
    if let Some(loc) = info.location() {
        kprintln!("  at {}:{}", loc.file(), loc.line());
    }
    arch::exit_qemu(false);
    arch::halt()
}
