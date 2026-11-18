//! The kernel's own GDT and TSS.
//!
//! The TSS does two jobs. `rsp0` is the kernel stack the CPU switches to when
//! an interrupt arrives while a program runs in user mode (ring 3); it is set
//! to the running thread's kernel stack on every switch. Interrupt stack
//! table entry 1 is a separate stack for double faults, so a kernel stack
//! overflow is reported instead of resetting the CPU.
//!
//! The segment order is fixed by `syscall`/`sysret`: kernel code, kernel
//! data, then (after one unused slot) user data and user code.

use core::arch::asm;
use core::mem::size_of;

pub const KERNEL_CS: u16 = 0x08;
pub const KERNEL_DS: u16 = 0x10;
/// `sysret` loads user SS from this + 8 and user CS from this + 16.
pub const SYSRET_BASE: u16 = 0x18;
pub const USER_DS: u16 = 0x20 | 3;
pub const USER_CS: u16 = 0x28 | 3;
const TSS_SEL: u16 = 0x30;

/// IST slot (1-based, as the IDT gate expects) used for double faults.
pub const DOUBLE_FAULT_IST: u8 = 1;

#[repr(C, packed)]
struct Tss {
    reserved0: u32,
    rsp: [u64; 3],
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap_base: u16,
}

#[repr(C, align(16))]
struct Stack([u8; 16384]);

static mut DOUBLE_FAULT_STACK: Stack = Stack([0; 16384]);

static mut TSS: Tss = Tss {
    reserved0: 0,
    rsp: [0; 3],
    reserved1: 0,
    ist: [0; 7],
    reserved2: 0,
    reserved3: 0,
    iomap_base: size_of::<Tss>() as u16,
};

const GDT_LEN: usize = 8;

static mut GDT: [u64; GDT_LEN] = [
    0,
    0x00AF_9A00_0000_FFFF, // 0x08 kernel code, 64-bit
    0x00CF_9200_0000_FFFF, // 0x10 kernel data
    0,                     // 0x18 unused (32-bit user code in other kernels)
    0x00CF_F200_0000_FFFF, // 0x20 user data, DPL 3
    0x00AF_FA00_0000_FFFF, // 0x28 user code, 64-bit, DPL 3
    0,                     // 0x30 TSS, low half
    0,                     // 0x38 TSS, high half
];

/// Sets the stack the CPU uses when an interrupt arrives in user mode.
pub fn set_kernel_stack(top: u64) {
    unsafe {
        let tss = core::ptr::addr_of_mut!(TSS);
        core::ptr::addr_of_mut!((*tss).rsp).cast::<u64>().write_unaligned(top);
    }
}

#[repr(C, packed)]
struct Pointer {
    limit: u16,
    base: u64,
}

pub fn init() {
    unsafe {
        let stack = core::ptr::addr_of!(DOUBLE_FAULT_STACK) as u64;
        let tss = &mut *core::ptr::addr_of_mut!(TSS);
        tss.ist[(DOUBLE_FAULT_IST - 1) as usize] = stack + size_of::<Stack>() as u64;

        let base = tss as *const Tss as u64;
        let limit = (size_of::<Tss>() - 1) as u64;
        let gdt = &mut *core::ptr::addr_of_mut!(GDT);
        gdt[6] = (limit & 0xFFFF)
            | ((base & 0xFF_FFFF) << 16)
            | (0x89 << 40) // present, available 64-bit TSS
            | (((limit >> 16) & 0xF) << 48)
            | (((base >> 24) & 0xFF) << 56);
        gdt[7] = base >> 32;

        let ptr = Pointer { limit: (size_of::<[u64; GDT_LEN]>() - 1) as u16, base: gdt.as_ptr() as u64 };
        asm!(
            "lgdt [{ptr}]",
            // reload CS with a far return
            "push {cs}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ds, {ds:x}",
            "mov es, {ds:x}",
            "mov ss, {ds:x}",
            "ltr {tss:x}",
            ptr = in(reg) &ptr,
            cs = in(reg) KERNEL_CS as u64,
            ds = in(reg) KERNEL_DS as u64,
            tss = in(reg) TSS_SEL as u64,
            tmp = out(reg) _,
        );
    }
}
