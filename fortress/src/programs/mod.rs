//! Built-in test programs for user mode: tiny assembly ones for x86_64
//! (test_programs.S) and Rust ones from the `user/` workspace, embedded as
//! ELF files built for the kernel's own processor.

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(include_str!("test_programs.S"));

#[cfg(target_arch = "x86_64")]
extern "C" {
    static prog_hello_start: u8;
    static prog_hello_end: u8;
    static prog_peek_start: u8;
    static prog_peek_end: u8;
    static prog_halt_start: u8;
    static prog_halt_end: u8;
    static prog_badptr_start: u8;
    static prog_badptr_end: u8;
    static prog_doubler_start: u8;
    static prog_doubler_end: u8;
    static prog_client_start: u8;
    static prog_client_end: u8;
}

#[cfg(target_arch = "x86_64")]
fn bytes(start: *const u8, end: *const u8) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(start, end as usize - start as usize) }
}

#[cfg(target_arch = "x86_64")]
pub fn hello() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_hello_start), core::ptr::addr_of!(prog_hello_end))
}

#[cfg(target_arch = "x86_64")]
pub fn peek() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_peek_start), core::ptr::addr_of!(prog_peek_end))
}

#[cfg(target_arch = "x86_64")]
pub fn halt() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_halt_start), core::ptr::addr_of!(prog_halt_end))
}

#[cfg(target_arch = "x86_64")]
pub fn badptr() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_badptr_start), core::ptr::addr_of!(prog_badptr_end))
}

#[cfg(target_arch = "x86_64")]
pub fn doubler() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_doubler_start), core::ptr::addr_of!(prog_doubler_end))
}

#[cfg(target_arch = "x86_64")]
pub fn client() -> &'static [u8] {
    bytes(core::ptr::addr_of!(prog_client_start), core::ptr::addr_of!(prog_client_end))
}

/// The first program written in Rust (user/hello-rust), as an ELF file.
pub fn hello_rust_elf() -> &'static [u8] {
    include_bytes!(env!("USER_ELF_HELLO_RUST"))
}

/// The console service (user/console), as an ELF file.
#[cfg(target_arch = "x86_64")]
pub fn console_elf() -> &'static [u8] {
    include_bytes!(env!("USER_ELF_CONSOLE"))
}

/// A program that prints through the console service (user/hello-console).
#[cfg(target_arch = "x86_64")]
pub fn hello_console_elf() -> &'static [u8] {
    include_bytes!(env!("USER_ELF_HELLO_CONSOLE"))
}

/// Sends a megabyte through a shared-memory ring (user/ring-send).
pub fn ring_send_elf() -> &'static [u8] {
    include_bytes!(env!("USER_ELF_RING_SEND"))
}

/// A program that reads kernel memory and must be stopped (user/crash).
pub fn crash_elf() -> &'static [u8] {
    include_bytes!(env!("USER_ELF_CRASH"))
}
