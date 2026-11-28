//! Interrupt descriptor table and CPU exception handling.
//!
//! All 256 vectors have a stub. A breakpoint is reported and execution
//! carries on; every other CPU exception prints the full register state and
//! stops the kernel with a panic. Vectors from 32 up are device interrupts:
//! the timer drives the scheduler, spurious ones are ignored.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};

global_asm!(include_str!("isr.S"), options(att_syntax));

extern "C" {
    static isr_table: [u64; 256];
}

/// Register state saved by `isr_common`, lowest address first.
#[repr(C)]
#[derive(Debug)]
pub struct Frame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub vector: u64,
    pub error: u64,
    // pushed by the CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Gate {
    offset_lo: u16,
    selector: u16,
    ist: u8,
    attrs: u8,
    offset_mid: u16,
    offset_hi: u32,
    reserved: u32,
}

impl Gate {
    const EMPTY: Gate = Gate { offset_lo: 0, selector: 0, ist: 0, attrs: 0, offset_mid: 0, offset_hi: 0, reserved: 0 };

    /// Present, ring 0, 64-bit interrupt gate (interrupts stay off in handlers).
    fn interrupt(handler: u64) -> Gate {
        Gate {
            offset_lo: handler as u16,
            selector: super::KERNEL_CS,
            ist: 0,
            attrs: 0x8E,
            offset_mid: (handler >> 16) as u16,
            offset_hi: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

static mut IDT: [Gate; 256] = [Gate::EMPTY; 256];

/// How many breakpoints have been handled; the boot self-test reads it.
pub static BREAKPOINTS: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    unsafe {
        let idt = &mut *core::ptr::addr_of_mut!(IDT);
        for (v, &handler) in isr_table.iter().enumerate() {
            idt[v] = Gate::interrupt(handler);
        }
        idt[8].ist = super::gdt::DOUBLE_FAULT_IST;
        let ptr = IdtPointer { limit: (core::mem::size_of::<[Gate; 256]>() - 1) as u16, base: idt.as_ptr() as u64 };
        asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
    }
}

const NAMES: [&str; 32] = [
    "divide error",
    "debug",
    "non-maskable interrupt",
    "breakpoint",
    "overflow",
    "bound range exceeded",
    "invalid opcode",
    "device not available",
    "double fault",
    "coprocessor segment overrun",
    "invalid TSS",
    "segment not present",
    "stack-segment fault",
    "general protection fault",
    "page fault",
    "reserved",
    "x87 floating-point error",
    "alignment check",
    "machine check",
    "SIMD floating-point error",
    "virtualization exception",
    "control protection exception",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "hypervisor injection exception",
    "VMM communication exception",
    "security exception",
    "reserved",
];

fn read_cr2() -> u64 {
    let v: u64;
    unsafe { asm!("mov {}, cr2", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[no_mangle]
extern "C" fn exception_dispatch(f: &mut Frame) {
    let v = f.vector as usize;
    if v >= 32 {
        super::interrupt(v);
        return;
    }
    if v == 3 {
        BREAKPOINTS.fetch_add(1, Ordering::Relaxed);
        crate::kprintln!("Breakpoint at {:#x}, continuing.", f.rip);
        return;
    }

    // From user mode: stop that program, not the system.
    if f.cs & 3 == 3 {
        let detail = if v == 14 { read_cr2() } else { f.error };
        crate::process::stop(NAMES[v], f.rip, detail);
    }

    crate::fbcon::resume();
    crate::kprintln!("\nCPU EXCEPTION {}: {} (error code {:#x})", v, NAMES[v], f.error);
    if v == 14 {
        let e = f.error;
        crate::kprintln!(
            "  page fault at {:#018x}: {} on {} page, {} mode",
            read_cr2(),
            if e & 2 != 0 { "write" } else { "read" },
            if e & 1 != 0 { "a protected" } else { "a missing" },
            if e & 4 != 0 { "user" } else { "kernel" },
        );
    }
    if v == 8 {
        let guard = super::paging::guard_page();
        if (guard..guard + 2 * 4096).contains(&f.rsp) {
            crate::kprintln!("  kernel stack overflow: stack pointer {:#x} reached the guard page", f.rsp);
        }
    }
    crate::kprintln!("  rip {:#018x}  cs {:#06x}  rflags {:#010x}", f.rip, f.cs, f.rflags);
    crate::kprintln!("  rsp {:#018x}  ss {:#06x}", f.rsp, f.ss);
    crate::kprintln!("  rax {:#018x}  rbx {:#018x}  rcx {:#018x}", f.rax, f.rbx, f.rcx);
    crate::kprintln!("  rdx {:#018x}  rsi {:#018x}  rdi {:#018x}", f.rdx, f.rsi, f.rdi);
    crate::kprintln!("  rbp {:#018x}  r8  {:#018x}  r9  {:#018x}", f.rbp, f.r8, f.r9);
    crate::kprintln!("  r10 {:#018x}  r11 {:#018x}  r12 {:#018x}", f.r10, f.r11, f.r12);
    crate::kprintln!("  r13 {:#018x}  r14 {:#018x}  r15 {:#018x}", f.r13, f.r14, f.r15);
    panic!("unhandled CPU exception {} ({})", v, NAMES[v]);
}
