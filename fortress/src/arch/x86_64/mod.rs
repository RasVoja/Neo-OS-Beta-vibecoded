//! x86_64 support: PVH entry, COM1 serial, halting, QEMU exit.

use core::arch::{asm, global_asm};

global_asm!(include_str!("boot.S"), options(att_syntax));

mod apic;
mod gdt;
mod idt;
pub mod paging;
mod syscall;

pub use syscall::enter_user;

pub use idt::BREAKPOINTS;

pub const NAME: &str = "x86_64";

use gdt::KERNEL_CS;

/// Sets up what the CPU needs before anything else can go wrong.
pub fn init_cpu() {
    gdt::init();
    idt::init();
    syscall::init();
}

/// Makes the next thread's address space and kernel stack current.
/// `root` is its program's top-level page table, or None for the kernel's.
pub fn prepare_thread(root: Option<u64>, kstack_top: u64) {
    paging::activate_root(root.unwrap_or_else(|| paging::kernel_space().root()));
    if kstack_top != 0 {
        syscall::set_kernel_stack(kstack_top);
    }
}

/// Switches to the kernel's own page tables; returns how much is mapped.
pub fn init_memory(ram_end: u64, kernel_end: u64) -> u64 {
    paging::init(ram_end, kernel_end)
}

/// Where the kernel heap lives in the address space (PML4 slot 288).
pub const HEAP_BASE: u64 = 0xFFFF_9000_0000_0000;

pub use paging::map_kernel_page;

pub use apic::{msi_message, HZ, USB_VECTOR};

/// Starts the periodic timer; returns the timer's input frequency in Hz.
pub fn init_timer() -> u64 {
    apic::init()
}

/// Lets the serial port interrupt when a byte arrives.
pub fn init_input() {
    apic::route_isa(4);
    apic::route_isa(1);
    apic::route_isa(12);
    unsafe {
        outb(COM1 + 4, 0x0B); // DTR + RTS + OUT2 (connects the IRQ line)
        outb(COM1 + 1, 0x01); // interrupt on received data
    }
}

/// Device interrupts (vector 32 and up), called from the IDT dispatcher.
fn interrupt(vector: usize) {
    match vector {
        apic::TIMER_VECTOR => {
            apic::eoi();
            crate::sched::timer_tick();
        }
        apic::SERIAL_VECTOR => {
            crate::input::irq();
            apic::eoi();
        }
        apic::KEYBOARD_VECTOR => {
            crate::irq::fire(1);
            apic::eoi();
        }
        apic::MOUSE_VECTOR => {
            crate::irq::fire(12);
            apic::eoi();
        }
        apic::USB_VECTOR => {
            crate::irq::fire(crate::irq::USB);
            apic::eoi();
        }
        apic::SPURIOUS_VECTOR => {} // LAPIC spurious: no EOI
        0xF0..=0xFE => {}           // stray legacy PIC interrupt
        v => {
            crate::kprintln!("Unexpected interrupt {}", v);
            apic::eoi();
        }
    }
}

/// Disables interrupts; returns whether they were on.
pub fn irq_save() -> bool {
    let flags: u64;
    unsafe { asm!("pushfq; pop {}; cli", out(reg) flags, options(nomem)) };
    flags & (1 << 9) != 0
}

pub fn irq_restore(were_on: bool) {
    if were_on {
        unsafe { asm!("sti", options(nomem, nostack)) };
    }
}

pub fn enable_interrupts() {
    unsafe { asm!("sti", options(nomem, nostack)) };
}

/// Enables interrupts and sleeps until the next one, atomically.
pub fn wait_for_interrupt() {
    unsafe { asm!("sti; hlt", options(nomem, nostack)) };
}

global_asm!(
    r#"
    .pushsection .text
    .global switch_context
    // switch_context(save_rsp: *mut u64, new_rsp: u64)
switch_context:
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov [rdi], rsp
    mov rsp, rsi
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

thread_trampoline:
    mov rdi, r12
    call thread_start
    ud2
    .popsection
"#
);

extern "C" {
    fn switch_context(save_rsp: *mut u64, new_rsp: u64);
    fn thread_trampoline();
}

/// Saves the current callee-saved state and stack in `*save_rsp`, then
/// continues the thread whose state is at `new_rsp`. Interrupts must be off.
///
/// # Safety
/// `new_rsp` must come from `init_thread_stack` or an earlier switch.
pub unsafe fn switch(save_rsp: *mut u64, new_rsp: u64) {
    switch_context(save_rsp, new_rsp);
}

/// Prepares a fresh stack so that the first switch to it runs
/// `crate::sched::thread_start(arg)`. Returns the initial stack pointer.
pub fn init_thread_stack(stack_top: u64, arg: u64) -> u64 {
    let top = stack_top & !0xF;
    let frame = (top - 8 - 6 * 8) as *mut u64;
    unsafe {
        // r15, r14, r13, r12 (= arg), rbx, rbp, return address
        let regs = [0, 0, 0, arg, 0, 0, thread_trampoline as *const () as u64];
        for (i, r) in regs.iter().enumerate() {
            frame.add(i).write(*r);
        }
    }
    frame as u64
}

/// Raises a breakpoint exception; the handler reports it and returns.
pub fn breakpoint() {
    unsafe { asm!("int3", options(nomem, nostack)) };
}

const COM1: u16 = 0x3F8;

pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

pub unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    asm!("in al, dx", out("al") v, in("dx") port, options(nomem, nostack, preserves_flags));
    v
}

pub unsafe fn outw(port: u16, val: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

pub unsafe fn inw(port: u16) -> u16 {
    let v: u16;
    asm!("in ax, dx", out("ax") v, in("dx") port, options(nomem, nostack, preserves_flags));
    v
}

pub unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
}

pub unsafe fn inl(port: u16) -> u32 {
    let v: u32;
    asm!("in eax, dx", out("eax") v, in("dx") port, options(nomem, nostack, preserves_flags));
    v
}

/// Reads a 32-bit register of PCI function (bus, dev, func) through the
/// legacy configuration ports.
pub fn pci_read32(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
    let addr = 0x8000_0000 | (bus as u32) << 16 | (dev as u32) << 11 | (func as u32) << 8 | (offset & 0xFC) as u32;
    unsafe {
        outl(0xCF8, addr);
        inl(0xCFC)
    }
}

pub fn pci_write32(bus: u8, dev: u8, func: u8, offset: u8, val: u32) {
    let addr = 0x8000_0000 | (bus as u32) << 16 | (dev as u32) << 11 | (func as u32) << 8 | (offset & 0xFC) as u32;
    unsafe {
        outl(0xCF8, addr);
        outl(0xCFC, val);
    }
}

/// Port I/O on behalf of a user-mode driver (checked by the caller).
/// `width` is 1, 2 or 4 bytes.
pub fn io_out(port: u16, val: u32, width: u64) {
    unsafe {
        match width {
            2 => outw(port, val as u16),
            4 => outl(port, val),
            _ => outb(port, val as u8),
        }
    }
}

pub fn io_in(port: u16, width: u64) -> u32 {
    unsafe {
        match width {
            2 => inw(port) as u32,
            4 => inl(port),
            _ => inb(port) as u32,
        }
    }
}

/// The serial port the console uses: base and number of registers.
pub const CONSOLE_IO: (u16, u16) = (COM1, 8);

/// 115200 baud, 8N1, FIFO on.
pub fn init_console() {
    unsafe {
        outb(COM1 + 1, 0x00); // interrupts off
        outb(COM1 + 3, 0x80); // DLAB on
        outb(COM1, 0x01); // divisor 1 = 115200
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03); // 8N1, DLAB off
        outb(COM1 + 2, 0xC7); // FIFO
        outb(COM1 + 4, 0x03); // DTR + RTS
    }
}

pub fn putc(b: u8) {
    unsafe {
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, b);
    }
}

/// The next byte typed on the serial console, if there is one.
pub fn getc() -> Option<u8> {
    unsafe { (inb(COM1 + 5) & 1 != 0).then(|| inb(COM1)) }
}

/// With `-device isa-debug-exit,iobase=0xf4,iosize=0x04` QEMU exits with
/// status (value << 1) | 1, so 0x10 -> 33 means success and 0x11 -> 35 failure.
/// Without that device this does nothing.
/// Waits roughly `us` microseconds: a write to port 0x80 (the POST code
/// port, unused) takes about a microsecond on PC hardware. For short device
/// waits, before or without the timer.
pub fn delay_us(us: u64) {
    for _ in 0..us {
        unsafe { outb(0x80, 0) };
    }
}

pub fn exit_qemu(success: bool) {
    unsafe { outb(0xf4, if success { 0x10 } else { 0x11 }) };
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack)) };
    }
}
