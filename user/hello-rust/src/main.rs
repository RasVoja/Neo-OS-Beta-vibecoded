//! The first Fortress program written in Rust. It runs in user mode, prints
//! through the kernel, works out a few numbers, reports one of them on the
//! port it was given (handle 0) and checks that what it was not given is
//! refused.

#![no_std]
#![no_main]

use neo_rt::{println, Error};

fn fib(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        (a, b) = (b, a + b);
    }
    a
}

#[no_mangle]
fn neo_main() -> i64 {
    println!("Hello from Rust in user mode!");
    let f = fib(50);
    println!("fib(50) = {}", f);

    // Not given: a receive right on handle 0, any handle 1, name lookup.
    if neo_rt::recv(0).err() != Some(Error::Denied) {
        return 101;
    }
    if neo_rt::send(1, 0, 0, 0) != Err(Error::BadHandle) {
        return 102;
    }
    if neo_rt::find_port("neo.test.user") != Err(Error::Denied) {
        return 103;
    }

    match neo_rt::send(0, 0x200, f, 50) {
        Ok(()) => 7,
        Err(_) => 104,
    }
}
