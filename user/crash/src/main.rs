//! A program that misbehaves on purpose: it reads kernel memory. The kernel
//! must stop it, and only it.

#![no_std]
#![no_main]

use neo_rt::println;

/// Kernel memory on both machines: mapped, but for the kernel only.
const KERNEL_ADDRESS: usize = 0x40_0000;

#[no_mangle]
fn neo_main() -> i64 {
    println!("crash: reading kernel memory...");
    let v = unsafe { core::ptr::read_volatile(KERNEL_ADDRESS as *const u64) };
    println!("crash: read {:#x}; this must not happen", v);
    v as i64
}
