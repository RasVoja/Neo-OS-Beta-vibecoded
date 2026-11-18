//! Echo server: answers every message on its port with the number doubled.
//! Plain neo-rt; on Genode the ports are the neo_ports server's.
//!
//! Handle 0 is its own port ("echo"), handle 1 the client's answer port
//! ("echo-reply"); both come from the component's config, the way Fortress
//! hands a program its handles when it starts it.

#![no_std]

use neo_rt::println;

const REQUEST: u64 = 1;
const ANSWER: u64 = 2;
const QUIT: u64 = 9;

#[no_mangle]
fn neo_main() -> i64 {
    let mut answered = 0u64;
    loop {
        let Ok(m) = neo_rt::recv(0) else { return 1 };
        match m.tag {
            REQUEST => {
                if neo_rt::send(1, ANSWER, m.a0 * 2, m.a1).is_err() {
                    return 2;
                }
                answered += 1;
            }
            QUIT => break,
            _ => return 3,
        }
    }
    println!("echo server: answered {} messages", answered);
    0
}
