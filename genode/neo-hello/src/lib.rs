//! The first Neo program on Genode: plain neo-rt calls, no Genode code.
//! The component in `../repos/neo/src/app/neo_hello` provides the calls.

#![no_std]

use neo_rt::println;

fn fib(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        (a, b) = (b, a + b);
    }
    a
}

#[no_mangle]
fn neo_main() -> i64 {
    println!("Hello from Rust on Genode!");
    println!("fib(50) = {}", fib(50));
    // Ports are a Fortress service the trial does not provide yet; the
    // program must get a clean error, not a crash.
    match neo_rt::find_port("display") {
        Err(neo_rt::Error::UnknownCall) => println!("find_port: not provided (as expected)"),
        other => {
            println!("find_port: unexpected {:?}", other);
            return 1;
        }
    }
    println!("neo_hello done");
    0
}
