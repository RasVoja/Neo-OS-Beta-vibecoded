//! AArch64 support (QEMU `virt` first): entry, exception vectors, the PL011
//! serial port, halting and leaving QEMU through semihosting.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};

global_asm!(include_str!("boot.S"));
global_asm!(include_str!("vectors.S"));

mod gic;
pub mod paging;

pub use gic::HZ;

pub const NAME: &str = "aarch64";

/// Where the kernel heap lives: the upper half, top-level slot 288 as on
/// x86_64, reached through TTBR1.
pub const HEAP_BASE: u64 = 0xFFFF_9000_0000_0000;

pub use paging::map_kernel_page;

/// Switches to the kernel's own page tables; returns how much is mapped.
pub fn init_memory(ram: &[(u64, u64)], kernel_start: u64, kernel_end: u64) -> u64 {
    paging::init(ram, kernel_start, kernel_end)
}

/// The PL011 UART of the QEMU `virt` machine.
const UART: usize = 0x0900_0000;
const UART_DR: usize = 0x00;
const UART_FR: usize = 0x18;
const FR_TXFF: u32 = 1 << 5;
const FR_RXFE: u32 = 1 << 4;

pub fn init_console() {
    // QEMU's PL011 works as it comes; real boards will need the clock and
    // baud rate set here.
}

pub fn putc(b: u8) {
    unsafe {
        while core::ptr::read_volatile((UART + UART_FR) as *const u32) & FR_TXFF != 0 {}
        core::ptr::write_volatile((UART + UART_DR) as *mut u32, b as u32);
    }
}

const UART_IMSC: usize = 0x38;
const UART_ICR: usize = 0x44;
/// Receive and receive-timeout interrupts.
const UART_RX_IRQS: u32 = (1 << 4) | (1 << 6);

/// Lets the UART interrupt when a byte arrives.
pub fn init_input() {
    unsafe { core::ptr::write_volatile((UART + UART_IMSC) as *mut u32, UART_RX_IRQS) };
    gic::enable_spi(gic::UART_INTID);
    for dev in crate::virtio_input::init() {
        let what = if dev.pointer { "Pointer" } else { "Keyboard" };
        match dev.result {
            Ok(()) => {
                gic::enable_spi(dev.intid);
                crate::kprintln!("{}: virtio-input (interrupt {})", what, dev.intid);
            }
            Err(e) => crate::kprintln!("{} found but not usable: {}", what, e),
        }
    }
}

fn uart_irq() {
    // Clear first, then drain: a byte that arrives after the drain raises
    // the interrupt again instead of being left unread.
    unsafe { core::ptr::write_volatile((UART + UART_ICR) as *mut u32, UART_RX_IRQS) };
    crate::input::irq();
}

/// The next byte typed on the serial console, if there is one.
pub fn getc() -> Option<u8> {
    unsafe {
        if core::ptr::read_volatile((UART + UART_FR) as *const u32) & FR_RXFE != 0 {
            return None;
        }
        Some(core::ptr::read_volatile((UART + UART_DR) as *const u32) as u8)
    }
}

/// Saved registers, as vectors.S lays them out.
#[repr(C)]
pub struct Frame {
    x: [u64; 31],
    elr: u64,
    spsr: u64,
    sp_el0: u64,
    _pad: [u64; 2],
    q: [u128; 32],
    fpsr: u64,
    fpcr: u64,
}

const _: () = assert!(core::mem::size_of::<Frame>() == 816);

pub static BREAKPOINTS: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    static exception_vectors: u8;
}

/// Sets up what the CPU needs before anything else can go wrong.
pub fn init_cpu() {
    unsafe {
        asm!(
            "msr vbar_el1, {v}",
            "isb",
            v = in(reg) core::ptr::addr_of!(exception_vectors) as u64,
            options(nostack)
        );
    }
}

fn exception_class_name(ec: u64) -> &'static str {
    match ec {
        0x00 => "unknown instruction",
        0x07 => "FP/SIMD access",
        0x15 => "system call",
        0x18 => "system register access",
        0x20 | 0x21 => "instruction abort",
        0x22 => "misaligned program counter",
        0x24 | 0x25 => "data abort",
        0x26 => "misaligned stack pointer",
        0x2F => "SError",
        0x3C => "breakpoint",
        _ => "other",
    }
}

const KIND_NAMES: [&str; 4] = ["synchronous", "IRQ", "FIQ", "SError"];
const FROM_NAMES: [&str; 4] = ["EL1 with SP_EL0", "EL1", "EL0 (AArch64)", "EL0 (AArch32)"];

#[no_mangle]
extern "C" fn exception(frame: &mut Frame, kind: u64) {
    let (esr, far): (u64, u64);
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) esr, options(nomem, nostack));
        asm!("mrs {}, far_el1", out(reg) far, options(nomem, nostack));
    }
    let ec = esr >> 26;
    // Interrupts, from the kernel (5) or from a program (9).
    if kind == 5 || kind == 9 {
        gic::handle_irq();
        return;
    }
    // From a program: a system call, or a fault that stops that program.
    if kind >= 8 {
        if kind == 8 && ec == 0x15 {
            let x = &frame.x;
            frame.x[0] = crate::syscall::syscall_dispatch(x[8], x[0], x[1], x[2], x[3]);
            return;
        }
        let detail = if ec == 0x20 || ec == 0x24 { far } else { esr };
        crate::process::stop(exception_class_name(ec), frame.elr, detail);
    }
    if kind == 4 && ec == 0x3C {
        BREAKPOINTS.fetch_add(1, Ordering::Relaxed);
        crate::kprintln!("Breakpoint at {:#x}", frame.elr);
        frame.elr += 4;
        return;
    }
    crate::kprintln!();
    crate::kprintln!(
        "CPU EXCEPTION: {} ({}) from {}",
        exception_class_name(ec),
        KIND_NAMES[(kind & 3) as usize],
        FROM_NAMES[(kind >> 2) as usize]
    );
    if ec == 0x24 || ec == 0x25 {
        // Data abort: the fault status code says what kind.
        let dfsc = esr & 0x3F;
        let what = match dfsc >> 2 {
            1 => "a missing",
            2 | 3 => "a protected",
            _ => "a bad",
        };
        crate::kprintln!(
            "  page fault at {:#018x}: {} on {} page, {} mode",
            far,
            if esr & (1 << 6) != 0 { "write" } else { "read" },
            what,
            if ec == 0x24 { "user" } else { "kernel" },
        );
        let guard = paging::guard_page();
        if (guard..guard + 4096).contains(&far) {
            crate::kprintln!("  kernel stack overflow: {:#x} is in the guard page", far);
        }
    }
    crate::kprintln!("  esr {:#010x}  far {:#018x}", esr, far);
    crate::kprintln!("  elr {:#018x}  spsr {:#010x}", frame.elr, frame.spsr);
    for i in 0..31 {
        crate::kprint!("  x{:<2} {:#018x}", i, frame.x[i]);
        if i % 4 == 3 || i == 30 {
            crate::kprintln!();
        }
    }
    panic!("unhandled CPU exception: {}", exception_class_name(ec));
}

/// Raises a breakpoint exception; the handler reports it and returns.
pub fn breakpoint() {
    unsafe { asm!("brk #0", options(nomem, nostack)) };
}

/// Runs an instruction that does not exist, to test fault reports.
pub fn undefined_instruction() {
    unsafe { asm!("udf #0", options(nomem, nostack)) };
}

/// Disables interrupts; returns whether they were on.
pub fn irq_save() -> bool {
    let daif: u64;
    unsafe {
        asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
        asm!("msr daifset, #2", options(nomem, nostack));
    }
    daif & (1 << 7) == 0
}

pub fn irq_restore(were_on: bool) {
    if were_on {
        unsafe { asm!("msr daifclr, #2", options(nomem, nostack)) };
    }
}

pub fn enable_interrupts() {
    unsafe { asm!("msr daifclr, #2", options(nomem, nostack)) };
}

/// Sleeps until the next interrupt and lets it in. WFI wakes up for a
/// pending interrupt even while they are masked, so none is missed.
pub fn wait_for_interrupt() {
    unsafe { asm!("wfi", "msr daifclr, #2", options(nomem, nostack)) };
}

/// Starts the periodic timer; returns the timer's input frequency in Hz.
pub fn init_timer() -> u64 {
    gic::init()
}

/// Makes the next thread's address space current. Its kernel stack needs
/// no setting: a thread in a program left EL1 from its own kernel stack, so
/// SP_EL1 already points there, and switch_context keeps it per thread.
pub fn prepare_thread(root: Option<u64>, _kstack_top: u64) {
    paging::activate_root(root.unwrap_or_else(|| paging::kernel_space().root()));
}

/// Leaves the kernel for good on this thread: runs the program at `entry`
/// in EL0 with its stack at `stack_top`, interrupts on and every register
/// cleared, so nothing of the kernel's leaks into it.
pub fn enter_user(entry: u64, stack_top: u64) -> ! {
    unsafe {
        asm!(
            "msr sp_el0, {stack}",
            "msr elr_el1, {entry}",
            "msr spsr_el1, xzr",        // EL0t, interrupts on
            ".irp n,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
            "movi v\\n\\().16b, #0",
            ".endr",
            "msr fpsr, xzr",
            ".irp n,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30",
            "mov x\\n, xzr",
            ".endr",
            "eret",
            stack = in(reg) stack_top,
            entry = in(reg) entry,
            options(noreturn)
        )
    }
}

global_asm!(
    r#"
    .pushsection .text
    .global switch_context
    // switch_context(save_sp: *mut u64, new_sp: u64): saves the registers a
    // function call must keep (x19-x30, d8-d15) on the stack, then resumes
    // the other thread from its stack.
switch_context:
    sub     sp, sp, #(20 * 8)
    stp     x19, x20, [sp, #(0 * 8)]
    stp     x21, x22, [sp, #(2 * 8)]
    stp     x23, x24, [sp, #(4 * 8)]
    stp     x25, x26, [sp, #(6 * 8)]
    stp     x27, x28, [sp, #(8 * 8)]
    stp     x29, x30, [sp, #(10 * 8)]
    stp     d8, d9, [sp, #(12 * 8)]
    stp     d10, d11, [sp, #(14 * 8)]
    stp     d12, d13, [sp, #(16 * 8)]
    stp     d14, d15, [sp, #(18 * 8)]
    mov     x9, sp
    str     x9, [x0]
    mov     sp, x1
    ldp     x19, x20, [sp, #(0 * 8)]
    ldp     x21, x22, [sp, #(2 * 8)]
    ldp     x23, x24, [sp, #(4 * 8)]
    ldp     x25, x26, [sp, #(6 * 8)]
    ldp     x27, x28, [sp, #(8 * 8)]
    ldp     x29, x30, [sp, #(10 * 8)]
    ldp     d8, d9, [sp, #(12 * 8)]
    ldp     d10, d11, [sp, #(14 * 8)]
    ldp     d12, d13, [sp, #(16 * 8)]
    ldp     d14, d15, [sp, #(18 * 8)]
    add     sp, sp, #(20 * 8)
    ret

thread_trampoline:
    mov     x0, x19
    bl      thread_start
    brk     #1
    .popsection
"#
);

extern "C" {
    fn switch_context(save_sp: *mut u64, new_sp: u64);
    fn thread_trampoline();
}

/// Saves the current callee-saved state and stack in `*save_sp`, then
/// continues the thread whose state is at `new_sp`. Interrupts must be off.
///
/// # Safety
/// `new_sp` must come from `init_thread_stack` or an earlier switch.
pub unsafe fn switch(save_sp: *mut u64, new_sp: u64) {
    switch_context(save_sp, new_sp);
}

/// Prepares a fresh stack so that the first switch to it runs
/// `crate::sched::thread_start(arg)`. Returns the initial stack pointer.
pub fn init_thread_stack(stack_top: u64, arg: u64) -> u64 {
    let frame = ((stack_top & !0xF) - 20 * 8) as *mut u64;
    unsafe {
        for i in 0..20 {
            frame.add(i).write(0);
        }
        frame.write(arg); // x19
        frame.add(11).write(thread_trampoline as *const () as u64); // x30
    }
    frame as u64
}

/// Ends the QEMU run through semihosting (needs `-semihosting`) with the
/// same exit statuses as on x86_64: 33 for success, 35 for failure.
pub fn exit_qemu(success: bool) {
    const SYS_EXIT: u64 = 0x18;
    const ADP_STOPPED_APPLICATION_EXIT: u64 = 0x20026;
    let block: [u64; 2] = [ADP_STOPPED_APPLICATION_EXIT, if success { 33 } else { 35 }];
    unsafe {
        asm!(
            "hlt #0xf000",
            in("x0") SYS_EXIT,
            in("x1") block.as_ptr(),
            options(nostack)
        );
    }
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("msr daifset, #0xf", "wfi", options(nomem, nostack)) };
    }
}
