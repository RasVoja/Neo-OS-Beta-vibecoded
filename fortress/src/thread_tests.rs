//! Self-tests for kernel threads, the timer and message ports. They need
//! only the scheduler, so every architecture with a timer runs them.

use super::*;
use core::sync::atomic::Ordering;
use sync::SpinLock;

static WORK: [core::sync::atomic::AtomicU64; 3] = [const { core::sync::atomic::AtomicU64::new(0) }; 3];
static NEXT_WORKER: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static STOP: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// A busy thread that never yields: only preemption lets others run.
fn worker() {
    let me = NEXT_WORKER.fetch_add(1, Ordering::Relaxed);
    while !STOP.load(Ordering::Relaxed) {
        WORK[me].fetch_add(1, Ordering::Relaxed);
    }
}

static PARKED_ID: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static WOKEN: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Parks twice; counts each wake-up.
fn parker() {
    PARKED_ID.store(sched::current_id(), Ordering::Relaxed);
    for _ in 0..2 {
        sched::park();
        WOKEN.fetch_add(1, Ordering::Relaxed);
    }
}

/// Waits up to `ticks` for `done` to hold.
pub fn wait_until(ticks: u64, done: impl Fn() -> bool) -> bool {
    for _ in 0..ticks {
        if done() {
            return true;
        }
        sched::sleep(1);
    }
    done()
}

const TAG_ADD: u64 = 1;
const TAG_QUIT: u64 = 2;
const MESSAGES: u64 = 1000;

/// A tiny service: answers "add" requests on the port named neo.test.adder.
fn adder_service() {
    let port = ipc::Port::new(4);
    ipc::publish("neo.test.adder", port.clone());
    loop {
        let m = port.recv();
        match m.tag {
            TAG_ADD => {
                if let Some(r) = m.reply {
                    r.send(ipc::Message::new(TAG_ADD, [m.args[0] + m.args[1], 0, 0]));
                }
            }
            TAG_QUIT => break,
            _ => {}
        }
    }
    ipc::withdraw("neo.test.adder");
}

static STREAM: SpinLock<Option<alloc::sync::Arc<ipc::Port>>> = SpinLock::new(None);

/// Sends 1..=MESSAGES through a small port, faster than they are read.
fn producer() {
    let port = STREAM.lock().clone().unwrap();
    for i in 1..=MESSAGES {
        port.send(ipc::Message::new(0, [i, 0, 0]));
    }
}

pub fn ipc_test() -> (bool, alloc::string::String) {
    let mut ok = true;

    // Streaming: a 4-slot port forces the producer to block when full and
    // the consumer (this thread) to block when empty. Order must hold.
    let port = ipc::Port::new(4);
    *STREAM.lock() = Some(port.clone());
    sched::spawn("producer", producer);
    let mut sum = 0;
    for i in 1..=MESSAGES {
        let m = port.recv();
        ok &= m.args[0] == i;
        sum += m.args[0];
    }
    ok &= sum == MESSAGES * (MESSAGES + 1) / 2;
    ok &= port.try_recv().is_none();
    *STREAM.lock() = None;

    // Request and reply through a port found by name.
    sched::spawn("adder", adder_service);
    let mut service = None;
    for _ in 0..10 {
        service = ipc::find("neo.test.adder");
        if service.is_some() {
            break;
        }
        sched::sleep(1);
    }
    let Some(adder) = service else {
        return (false, alloc::string::String::from("adder service not found"));
    };
    ok &= !ipc::publish("neo.test.adder", ipc::Port::new(1)); // name taken
    let mut calls = 0;
    for a in 0..20u64 {
        let r = adder.call(ipc::Message::new(TAG_ADD, [a, 1000, 0]));
        ok &= r.args[0] == a + 1000;
        calls += 1;
    }
    adder.send(ipc::Message::new(TAG_QUIT, [0; 3]));
    drop(adder);
    sched::sleep(2);
    ok &= ipc::find("neo.test.adder").is_none();

    (ok, alloc::format!("{} streamed in order, {} calls answered", MESSAGES, calls))
}

pub fn sleep_test() -> (bool, alloc::string::String) {
    use sched::{IDLE_RUNS, TICKS};

    // Sleeping: nothing else is ready, so the idle thread must run.
    let idle_before = IDLE_RUNS.load(Ordering::Relaxed);
    let t0 = TICKS.load(Ordering::Relaxed);
    sched::sleep(10);
    let slept = TICKS.load(Ordering::Relaxed) - t0;
    let idled = IDLE_RUNS.load(Ordering::Relaxed) - idle_before;
    let mut ok = (10..=13).contains(&slept) && idled >= 1;

    // Yielding with nobody else ready (only idle) keeps this thread running.
    let switches = sched::SWITCHES.load(Ordering::Relaxed);
    sched::yield_now();
    ok &= sched::SWITCHES.load(Ordering::Relaxed) == switches;

    // Park and unpark: the first unpark wakes a parked thread; the second
    // arrives before the thread parks again and must not be lost.
    sched::spawn("parker", parker);
    sched::sleep(3);
    let id = PARKED_ID.load(Ordering::Relaxed);
    ok &= id != 0 && WOKEN.load(Ordering::Relaxed) == 0;
    sched::unpark(id);
    sched::sleep(3);
    ok &= WOKEN.load(Ordering::Relaxed) == 1;
    sched::unpark(id);
    sched::sleep(3);
    ok &= WOKEN.load(Ordering::Relaxed) == 2;

    (ok, alloc::format!("slept {} ticks, idle ran {} times", slept, idled))
}

pub fn sched_test() -> (bool, alloc::string::String) {
    use sched::{SWITCHES, TICKS};
    let start = TICKS.load(Ordering::Relaxed);
    let switches = SWITCHES.load(Ordering::Relaxed);
    let threads_before = sched::thread_count();
    for name in ["worker-a", "worker-b", "worker-c"] {
        sched::spawn(name, worker);
    }
    // Let the workers run for about 0.3 s; this thread sleeps meanwhile.
    sched::sleep(30);
    STOP.store(true, Ordering::Relaxed);
    // Wait (at most ~1 s) for the workers to end and be cleaned up.
    let deadline = TICKS.load(Ordering::Relaxed) + 100;
    while sched::thread_count() > threads_before && TICKS.load(Ordering::Relaxed) < deadline {
        sched::sleep(1);
    }
    let ticks = TICKS.load(Ordering::Relaxed) - start;
    let switched = SWITCHES.load(Ordering::Relaxed) - switches;
    let all_ran = WORK.iter().all(|w| w.load(Ordering::Relaxed) > 0);
    let ok = all_ran && switched >= 6 && sched::thread_count() == threads_before;
    (ok, alloc::format!("3 threads, {} switches in {} ticks", switched, ticks))
}

static FP_RESULTS: [core::sync::atomic::AtomicU64; 2] = [const { core::sync::atomic::AtomicU64::new(0) }; 2];
static FP_NEXT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
const FP_ROUNDS: u32 = 300_000;

/// Floating-point work whose result depends on every step.
fn fp_work(seed: f64) -> u64 {
    let mut x = seed;
    let mut v = [seed; 4];
    for i in 0..FP_ROUNDS {
        x = x * 1.000_000_1 + core::hint::black_box(0.5);
        for (j, e) in v.iter_mut().enumerate() {
            *e = *e * 0.999_9 + (i as f64) * (j as f64 + 1.0);
        }
    }
    (x + v.iter().sum::<f64>()).to_bits()
}

fn fp_worker() {
    let me = FP_NEXT.fetch_add(1, Ordering::Relaxed);
    FP_RESULTS[me].store(fp_work(me as f64 + 1.0), Ordering::Relaxed);
}

/// Two threads do floating-point work while the timer preempts them; their
/// results must match the same work done without interruption. This checks
/// that interrupts and thread switches keep the FP/SIMD registers.
pub fn fp_test() -> (bool, alloc::string::String) {
    let want = [fp_work(1.0), fp_work(2.0)];
    let before = sched::SWITCHES.load(Ordering::Relaxed);
    let threads = sched::thread_count();
    for name in ["fp-a", "fp-b"] {
        sched::spawn(name, fp_worker);
    }
    let done = wait_until(500, || sched::thread_count() == threads);
    let got = [FP_RESULTS[0].load(Ordering::Relaxed), FP_RESULTS[1].load(Ordering::Relaxed)];
    let switches = sched::SWITCHES.load(Ordering::Relaxed) - before;
    (
        done && got == want && switches >= 4,
        alloc::format!("2 threads, {} switches, results {}", switches, if got == want { "match" } else { "DIFFER" }),
    )
}
