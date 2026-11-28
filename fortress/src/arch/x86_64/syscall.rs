//! Entering user mode and coming back through `syscall`.
//!
//! A program calls the kernel with the `syscall` instruction: number in rax,
//! arguments in rdi, rsi, rdx, r10; the result comes back in rax. rcx and r11
//! are destroyed by the instruction itself; rdi, rsi, rdx and r8-r10 come back
//! as zero, so no kernel value leaks to the program.
//!
//! On entry the CPU does not switch stacks, so the entry code moves to the
//! running thread's kernel stack (`SYSCALL_STACK`, updated on every thread
//! switch) before anything else. Interrupts stay off until it has.

use super::gdt;
use core::arch::{asm, global_asm};

const EFER: u32 = 0xC000_0080;
const STAR: u32 = 0xC000_0081;
const LSTAR: u32 = 0xC000_0082;
const FMASK: u32 = 0xC000_0084;

/// Kernel stack top of the running thread, read by `syscall_entry`.
#[no_mangle]
static mut SYSCALL_STACK: u64 = 0;
#[no_mangle]
static mut SYSCALL_USER_RSP: u64 = 0;

global_asm!(
    r#"
    .pushsection .text
    .global syscall_entry
syscall_entry:
    mov [rip + SYSCALL_USER_RSP], rsp
    mov rsp, [rip + SYSCALL_STACK]
    push qword ptr [rip + SYSCALL_USER_RSP]
    push rcx                    // user rip
    push r11                    // user rflags
    sub rsp, 8                  // keep the stack 16-byte aligned
    mov r8, r10                 // syscall_dispatch(num, a0, a1, a2, a3)
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax
    call syscall_dispatch
    xor edi, edi                // do not hand kernel values back
    xor esi, esi
    xor edx, edx
    xor r8d, r8d
    xor r9d, r9d
    xor r10d, r10d
    add rsp, 8
    pop r11
    pop rcx
    pop rsp
    sysretq
    .popsection
"#
);

extern "C" {
    fn syscall_entry();
}

fn wrmsr(msr: u32, v: u64) {
    unsafe { asm!("wrmsr", in("ecx") msr, in("eax") v as u32, in("edx") (v >> 32) as u32, options(nomem, nostack)) };
}

fn rdmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    unsafe { asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nomem, nostack)) };
    (hi as u64) << 32 | lo as u64
}

pub fn init() {
    wrmsr(EFER, rdmsr(EFER) | 1); // SCE: syscall/sysret on
                                  // The RPL (3) goes into the selector base: Intel makes the user selectors
                                  // RPL 3 on `sysret` by itself, AMD CPUs do not (SS would stay 0x20), and the
                                  // next interrupt would then fail to `iretq` with #GP(0x20) on a real PC.
    wrmsr(STAR, ((gdt::SYSRET_BASE | 3) as u64) << 48 | (gdt::KERNEL_CS as u64) << 32);
    wrmsr(LSTAR, syscall_entry as *const () as u64);
    wrmsr(FMASK, 0x700); // clear TF, IF and DF on entry
}

/// Where interrupts and system calls from user mode land: the top of the
/// running thread's kernel stack.
pub fn set_kernel_stack(top: u64) {
    gdt::set_kernel_stack(top);
    unsafe { SYSCALL_STACK = top };
}

/// Leaves the kernel for good on this thread: jumps to `entry` in ring 3
/// with the stack at `stack_top` and interrupts on.
pub fn enter_user(entry: u64, stack_top: u64) -> ! {
    unsafe {
        asm!(
            "push {ss}",
            "push {sp}",
            "push 0x202",       // rflags: IF
            "push {cs}",
            "push {ip}",
            "xor eax, eax",
            "xor ebx, ebx",
            "xor ecx, ecx",
            "xor edx, edx",
            "xor esi, esi",
            "xor edi, edi",
            "xor ebp, ebp",
            "xor r8d, r8d",
            "xor r9d, r9d",
            "xor r10d, r10d",
            "xor r11d, r11d",
            "xor r12d, r12d",
            "xor r13d, r13d",
            "xor r14d, r14d",
            "xor r15d, r15d",
            "iretq",
            ss = in(reg) gdt::USER_DS as u64,
            sp = in(reg) stack_top,
            cs = in(reg) gdt::USER_CS as u64,
            ip = in(reg) entry,
            options(noreturn),
        )
    }
}
