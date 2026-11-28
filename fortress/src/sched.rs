//! Kernel threads and a preemptive round-robin scheduler.
//!
//! Every thread has its own 16 KiB stack from the heap. The timer interrupt
//! calls `timer_tick`, which switches to the next ready thread every
//! `SLICE_TICKS` ticks; a thread can also give up the CPU with `yield_now`.
//! When a thread was preempted, its interrupt frame stays on its own stack
//! and the interrupt returns normally once the thread is picked again.
//!
//! The thread that ran `kmain` becomes thread 0, the boot thread. An idle
//! thread runs only when nothing else is ready; it halts the CPU until the
//! next interrupt.
//!
//! Threads can sleep for a number of ticks, or park until another thread
//! unparks them. An unpark that arrives before the park is remembered, so a
//! wake-up is never lost.

use crate::arch;
use crate::process::Process;
use crate::sync::SpinLock;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

const STACK_SIZE: usize = 16 * 1024;
/// Timer ticks a thread may run before it is preempted.
const SLICE_TICKS: u64 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Ready,
    Running,
    /// Sleeping until the given tick.
    Sleeping(u64),
    /// Parked until `unpark`.
    Blocked,
    Dead,
}

struct Thread {
    id: u64,
    name: &'static str,
    rsp: u64,
    state: State,
    /// An unpark arrived while the thread was not parked.
    unpark_pending: bool,
    /// Top of the thread's kernel stack (0 for the boot thread).
    kstack_top: u64,
    /// Set when the thread runs a program in user mode.
    process: Option<Box<Process>>,
    _stack: Option<Box<[u8]>>,
}

struct Scheduler {
    threads: Vec<Box<Thread>>,
    current: usize,
    next_id: u64,
    slice_left: u64,
    idle_id: u64,
}

static SCHED: SpinLock<Option<Scheduler>> = SpinLock::new(None);

pub static TICKS: AtomicU64 = AtomicU64::new(0);
pub static SWITCHES: AtomicU64 = AtomicU64::new(0);
/// How many times the idle thread has been switched to.
pub static IDLE_RUNS: AtomicU64 = AtomicU64::new(0);
/// Timer ticks that found the idle thread running: the rest is work.
pub static IDLE_TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let boot = Box::new(Thread {
        id: 0,
        name: "boot",
        rsp: 0,
        state: State::Running,
        unpark_pending: false,
        kstack_top: 0,
        process: None,
        _stack: None,
    });
    *SCHED.lock() = Some(Scheduler {
        threads: alloc::vec![boot],
        current: 0,
        next_id: 1,
        slice_left: SLICE_TICKS,
        idle_id: u64::MAX,
    });
    let idle = spawn("idle", idle);
    SCHED.lock().as_mut().unwrap().idle_id = idle;
}

fn idle() {
    loop {
        arch::wait_for_interrupt();
    }
}

/// Starts a new kernel thread running `entry`. Returns its id.
pub fn spawn(name: &'static str, entry: fn()) -> u64 {
    spawn_inner(name, entry, None)
}

/// Starts a thread that owns `process`; `entry` runs in the kernel first
/// (with the process's address space active) and drops to user mode.
pub fn spawn_process(name: &'static str, process: Process, entry: fn()) -> u64 {
    spawn_inner(name, entry, Some(Box::new(process)))
}

/// Runs `f` on the calling thread's process, if it has one.
pub fn with_current_process<R>(f: impl FnOnce(&mut Process) -> R) -> Option<R> {
    let mut guard = SCHED.lock();
    let s = guard.as_mut()?;
    let cur = s.current;
    s.threads[cur].process.as_deref_mut().map(f)
}

fn spawn_inner(name: &'static str, entry: fn(), process: Option<Box<Process>>) -> u64 {
    let stack: Box<[u8]> = alloc::vec![0u8; STACK_SIZE].into_boxed_slice();
    let top = stack.as_ptr() as u64 + STACK_SIZE as u64;
    let rsp = arch::init_thread_stack(top, entry as *const () as u64);
    let mut guard = SCHED.lock();
    let s = guard.as_mut().expect("scheduler not started");
    let id = s.next_id;
    s.next_id += 1;
    s.threads.push(Box::new(Thread {
        id,
        name,
        rsp,
        state: State::Ready,
        unpark_pending: false,
        kstack_top: top & !0xF,
        process,
        _stack: Some(stack),
    }));
    id
}

/// Id of the calling thread.
pub fn current_id() -> u64 {
    SCHED.lock().as_ref().map_or(0, |s| s.threads[s.current].id)
}

/// Sleeps for at least `ticks` timer ticks (1 tick = 1/HZ s).
pub fn sleep(ticks: u64) {
    let irq = arch::irq_save();
    {
        let mut guard = SCHED.lock();
        let s = guard.as_mut().expect("scheduler not started");
        let cur = s.current;
        s.threads[cur].state = State::Sleeping(TICKS.load(Ordering::Relaxed) + ticks);
    }
    schedule();
    arch::irq_restore(irq);
}

/// Blocks the calling thread until another thread calls `unpark` on it.
/// Returns at once if an unpark already arrived.
pub fn park() {
    let irq = arch::irq_save();
    let block = {
        let mut guard = SCHED.lock();
        let s = guard.as_mut().expect("scheduler not started");
        let cur = s.current;
        let t = &mut s.threads[cur];
        if t.unpark_pending {
            t.unpark_pending = false;
            false
        } else {
            t.state = State::Blocked;
            true
        }
    };
    if block {
        schedule();
    }
    arch::irq_restore(irq);
}

/// Wakes a parked thread, or makes its next `park` return at once.
pub fn unpark(id: u64) {
    let mut guard = SCHED.lock();
    let Some(s) = guard.as_mut() else { return };
    if let Some(t) = s.threads.iter_mut().find(|t| t.id == id) {
        match t.state {
            State::Blocked => t.state = State::Ready,
            State::Dead => {}
            _ => t.unpark_pending = true,
        }
    }
}

/// First Rust code a new thread runs (called from the arch trampoline).
#[no_mangle]
extern "C" fn thread_start(entry: u64) -> ! {
    arch::enable_interrupts();
    let f: fn() = unsafe { core::mem::transmute(entry as *const ()) };
    f();
    exit()
}

/// Ends the calling thread.
pub fn exit() -> ! {
    let irq = arch::irq_save();
    {
        let mut guard = SCHED.lock();
        let s = guard.as_mut().unwrap();
        let cur = s.current;
        s.threads[cur].state = State::Dead;
    }
    schedule();
    arch::irq_restore(irq);
    unreachable!("a dead thread was scheduled again");
}

/// Gives the CPU to the next ready thread, if there is one.
pub fn yield_now() {
    let irq = arch::irq_save();
    schedule();
    arch::irq_restore(irq);
}

/// Called by the timer interrupt (interrupts off).
pub fn timer_tick() {
    let now = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    let due = {
        let mut guard = SCHED.lock();
        match guard.as_mut() {
            Some(s) => {
                let mut woke = false;
                for t in s.threads.iter_mut() {
                    if let State::Sleeping(until) = t.state {
                        if until <= now {
                            t.state = State::Ready;
                            woke = true;
                        }
                    }
                }
                s.slice_left = s.slice_left.saturating_sub(1);
                // Leave idle at once when someone woke up.
                let idle = s.threads[s.current].id == s.idle_id;
                if idle {
                    IDLE_TICKS.fetch_add(1, Ordering::Relaxed);
                }
                s.slice_left == 0 || (woke && idle)
            }
            None => false,
        }
    };
    if due {
        schedule();
    }
}

/// Picks the next ready thread round-robin and switches to it.
/// Must be called with interrupts off.
fn schedule() {
    let (save, next) = {
        let mut guard = SCHED.lock();
        let Some(s) = guard.as_mut() else { return };
        s.slice_left = SLICE_TICKS;

        // Free the stacks of threads that ended, except the current one,
        // which is still running on its stack.
        let cur_id = s.threads[s.current].id;
        s.threads.retain(|t| t.state != State::Dead || t.id == cur_id);
        s.current = s.threads.iter().position(|t| t.id == cur_id).unwrap();

        let n = s.threads.len();
        let cur = s.current;
        let idle_id = s.idle_id;
        // Round-robin over ready threads; idle only when nobody else is ready.
        let ready = |i: &usize| s.threads[*i].state == State::Ready;
        let candidates = (1..=n).map(|i| (cur + i) % n);
        let Some(next) = candidates.clone().filter(ready).find(|&i| s.threads[i].id != idle_id).or_else(|| {
            // The current thread may keep running if it still can.
            if s.threads[cur].state == State::Running {
                Some(cur)
            } else {
                candidates.filter(ready).next()
            }
        }) else {
            return; // nothing else to run: keep going
        };
        if next == cur {
            return;
        }
        if s.threads[cur].state == State::Running {
            s.threads[cur].state = State::Ready;
        }
        s.threads[next].state = State::Running;
        s.current = next;
        if s.threads[next].id == idle_id {
            IDLE_RUNS.fetch_add(1, Ordering::Relaxed);
        }
        let save = &mut s.threads[cur].rsp as *mut u64;
        let t = &s.threads[next];
        arch::prepare_thread(t.process.as_ref().map(|p| p.root()), t.kstack_top);
        (save, t.rsp)
    };
    // The thread boxes do not move, so `save` stays valid after unlocking.
    SWITCHES.fetch_add(1, Ordering::Relaxed);
    unsafe { arch::switch(save, next) };
}

/// Ids and names of live threads (for diagnostics).
pub fn thread_count() -> usize {
    SCHED.lock().as_ref().map_or(0, |s| s.threads.len())
}

pub fn current_name() -> &'static str {
    SCHED.lock().as_ref().map_or("?", |s| s.threads[s.current].name)
}
