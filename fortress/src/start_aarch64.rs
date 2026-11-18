//! The AArch64 boot path (QEMU `virt`). Younger than the x86_64 one: it
//! finds RAM in the device tree, builds the kernel's page tables, starts the
//! timer and the scheduler, runs Rust programs in EL0 and the same
//! self-tests as x86_64 (except the x86-only assembly programs and the
//! console service).

use super::*;
use core::sync::atomic::Ordering;

/// QEMU passes the device tree in x0 when it boots a Linux-style image; for
/// an ELF file it leaves it at the start of RAM.
const DTB_AT_RAM_START: u64 = 0x4000_0000;

#[no_mangle]
pub extern "C" fn kmain(dtb: u64, start_el: u64) -> ! {
    arch::init_console();
    kprintln!("Neo Boing Platinum OS Lite: Fortress {} ({})", VERSION, arch::NAME);
    arch::init_cpu();
    kprintln!("Running in EL1 (started in EL{})", start_el);
    kprintln!("Exception handlers installed.");

    let tree = unsafe { fdt::DeviceTree::from_addr(dtb).or_else(|| fdt::DeviceTree::from_addr(DTB_AT_RAM_START)) };
    let Some(tree) = tree else {
        kprintln!("No device tree found; cannot tell where RAM is.");
        arch::exit_qemu(false);
        arch::halt()
    };
    let Some(info) = tree.boot_info() else {
        kprintln!("The device tree is damaged.");
        arch::exit_qemu(false);
        arch::halt()
    };
    let (ts, te) = tree.range();
    kprintln!("Device tree at {:#x} ({} bytes)", ts, te - ts);

    let mut usable = 0;
    for r in info.memory() {
        kprintln!("  mem {:#012x}..{:#012x} usable", r.addr, r.addr + r.size);
        usable += r.size;
    }
    for r in info.reserved() {
        kprintln!("  mem {:#012x}..{:#012x} reserved", r.addr, r.addr + r.size);
    }
    kprintln!("Usable RAM: {} MiB", usable / (1024 * 1024));
    init_frames(&info, (ts, te));

    let mut ram = [(0u64, 0u64); fdt::MAX_REGIONS];
    for (slot, r) in ram.iter_mut().zip(info.memory()) {
        *slot = (r.addr, r.addr + r.size);
    }
    let (ks, ke) = kernel_image();
    let mapped = arch::init_memory(&ram[..info.memory_count], ks, ke);
    mem::frame::FRAMES_ALLOC.lock().set_limit(mapped);
    kprintln!(
        "Kernel page tables active: {} GiB identity mapped, stack guard page at {:#x}",
        mapped >> 30,
        arch::paging::guard_page()
    );

    init_bootfs(info.initrd);

    init_disk();

    if let Some(fb) = display::find() {
        kprintln!("Screen: {} {}x{} at {:#x}", display::KIND, display::WIDTH, display::HEIGHT, fb);
    }

    sched::init();
    let timer_hz = arch::init_timer();
    arch::init_input();
    arch::enable_interrupts();
    kprintln!("Timer running at {} Hz (timer clock {} MHz); scheduler started", arch::HZ, timer_hz / 1_000_000);

    if !info.cmdline.is_empty() {
        kprintln!("Command line: {}", info.cmdline);
    }
    kprintln!("Fortress is up.");

    for word in info.cmdline.split(' ') {
        match word {
            "neo.test" => self_test(),
            "neo.shell" => run_shell(),
            "neo.test=undefined" => arch::undefined_instruction(),
            "neo.test=pagefault" => provoke_page_fault(),
            "neo.test=stackoverflow" => {
                overflow_stack(0);
            }
            _ => {}
        }
    }
    arch::halt()
}

/// Hands RAM to the frame allocator, minus the kernel, the device tree and
/// what the tree reserves. The bitmap goes right after the kernel.
fn init_frames(info: &fdt::BootInfo, tree: (u64, u64)) {
    let end = info.memory().iter().map(|r| r.addr + r.size).max().unwrap_or(0);
    let bytes = mem::frame::FrameAllocator::bitmap_bytes(end);
    let (ks, ke) = kernel_image();
    let at = ke.next_multiple_of(mem::PAGE_SIZE);
    assert!(at + bytes <= end, "no room for the frame bitmap");

    let mut fa = mem::frame::FRAMES_ALLOC.lock();
    unsafe { fa.init(at, end) };
    for r in info.memory() {
        fa.add_free(r.addr, r.addr + r.size);
    }
    fa.reserve(ks, ke);
    fa.reserve(tree.0, tree.1);
    if let Some((s, e)) = info.initrd {
        fa.reserve(s, e);
    }
    for r in info.reserved() {
        fa.reserve(r.addr, r.addr + r.size);
    }
    fa.reserve(at, at + bytes);
    // The boot page tables map RAM up to 4 GiB.
    fa.set_limit(end.min(1 << 32));
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

    let (sc, report) = thread_tests::sched_test();
    kprintln!("Self-test: scheduler {} ({})", if sc { "ok" } else { "FAILED" }, report);

    let (sl, report) = thread_tests::sleep_test();
    kprintln!("Self-test: sleep and park {} ({})", if sl { "ok" } else { "FAILED" }, report);

    let (ip, report) = thread_tests::ipc_test();
    kprintln!("Self-test: message ports {} ({})", if ip { "ok" } else { "FAILED" }, report);

    let (fp, report) = thread_tests::fp_test();
    kprintln!("Self-test: FP registers across switches {} ({})", if fp { "ok" } else { "FAILED" }, report);

    let (el, report) = program_tests::elf_test();
    kprintln!("Self-test: Rust programs {} ({})", if el { "ok" } else { "FAILED" }, report);

    let (bf, report) = program_tests::bootfs_test();
    kprintln!("Self-test: boot image {} ({})", if bf { "ok" } else { "FAILED" }, report);

    let (dk, report) = program_tests::disk_test();
    kprintln!("Self-test: disk {} ({})", if dk { "ok" } else { "FAILED" }, report);

    let (cr, report) = program_tests::crash_test();
    kprintln!("Self-test: faulty program stopped {} ({})", if cr { "ok" } else { "FAILED" }, report);

    let (dp, report) = display::self_test();
    kprintln!("Self-test: display service {} ({})", if dp { "ok" } else { "FAILED" }, report);

    let (wn, report) = display::window_test();
    kprintln!("Self-test: a program's window {} ({})", if wn { "ok" } else { "FAILED" }, report);

    let (tm, report) = display::terminal_test();
    kprintln!("Self-test: terminal window {} ({})", if tm { "ok" } else { "FAILED" }, report);

    let (rg, report) = crate::ring_tests::ring_test();
    kprintln!("Self-test: shared-memory ring {} ({})", if rg { "ok" } else { "FAILED" }, report);

    let (ly, report) = display::layout_test();
    kprintln!("Self-test: layout switch on the Bar {} ({})", if ly { "ok" } else { "FAILED" }, report);

    arch::exit_qemu(bp && fr && pg && hp && sc && sl && ip && fp && el && bf && dk && cr && dp && wn && tm && ly && rg);
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
        && space.translate(0x0900_0000) == Some(0x0900_0000) // the UART
}

/// Reads memory that is not mapped, to check that the fault is reported.
fn provoke_page_fault() {
    // The kernel page tables map 4 GiB (or all RAM); this is far above.
    let p = 0x40_0000_0000usize as *const u64;
    let v = unsafe { core::ptr::read_volatile(p) };
    kprintln!("Read {:#x} from unmapped memory; this should not happen.", v);
}
