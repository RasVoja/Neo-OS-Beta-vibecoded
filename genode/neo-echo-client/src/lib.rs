//! Echo client: finds the "echo" port by name, sends it numbers and checks
//! each answer. Plain neo-rt; on Genode the ports are the neo_ports
//! server's. Handle 0 is its answer port ("echo-reply"), from its config.

#![no_std]

use neo_rt::println;

const REQUEST: u64 = 1;
const ANSWER: u64 = 2;
const QUIT: u64 = 9;
const COUNT: u64 = 100;

#[no_mangle]
fn neo_main() -> i64 {
    // The server makes its port when it starts; wait for it a little.
    let mut echo = neo_rt::find_port("echo");
    for _ in 0..100 {
        if echo.is_ok() {
            break;
        }
        neo_rt::sleep(1);
        echo = neo_rt::find_port("echo");
    }
    let Ok(echo) = echo else {
        println!("echo client: no \"echo\" port");
        return 1;
    };
    // An unnamed port and an empty queue behave as on Fortress.
    let Ok(spare) = neo_rt::new_port(4) else { return 2 };
    if !matches!(neo_rt::try_recv(spare), Ok(None)) {
        return 3;
    }
    let mut right = 0;
    for i in 1..=COUNT {
        if neo_rt::send(echo, REQUEST, i, 0).is_err() {
            return 4;
        }
        match neo_rt::recv(0) {
            Ok(m) if m.tag == ANSWER && m.a0 == 2 * i => right += 1,
            _ => {}
        }
    }
    let _ = neo_rt::send(echo, QUIT, 0, 0);
    let _ = neo_rt::close(spare);
    println!("echo client: {} of {} answers right", right, COUNT);
    if right == COUNT {
        0
    } else {
        5
    }
}
