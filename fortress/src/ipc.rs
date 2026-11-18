//! Message ports: the first seed of Exec-N.
//!
//! A port is a bounded queue of messages between threads. `send` blocks
//! while the port is full and `recv` blocks while it is empty; blocked
//! threads park and are unparked by the other side, so nobody spins.
//! Ports can be published under a name so that a program can find a
//! service without knowing which thread runs it, as on the Amiga.
//!
//! Later, Exec-N moves messages between address spaces over shared-memory
//! rings. This version works between kernel threads only.

use crate::sched;
use crate::sync::SpinLock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// What travels through a port: a tag saying what is asked, a few words of
/// payload and, optionally, the port where the answer should go.
pub struct Message {
    pub tag: u64,
    pub args: [u64; 3],
    pub reply: Option<Arc<Port>>,
    /// A capability passed along with the message (shared memory, say).
    pub cap: Option<crate::process::Cap>,
}

impl Message {
    pub fn new(tag: u64, args: [u64; 3]) -> Self {
        Message { tag, args, reply: None, cap: None }
    }
}

struct Inner {
    queue: VecDeque<Message>,
    capacity: usize,
    /// Threads waiting for room (senders) or for a message (receivers).
    senders: Vec<u64>,
    receivers: Vec<u64>,
}

pub struct Port {
    inner: SpinLock<Inner>,
}

fn wake_one(waiters: &mut Vec<u64>) {
    if !waiters.is_empty() {
        sched::unpark(waiters.remove(0));
    }
}

fn enlist(waiters: &mut Vec<u64>, me: u64) {
    if !waiters.contains(&me) {
        waiters.push(me);
    }
}

impl Port {
    pub fn new(capacity: usize) -> Arc<Port> {
        assert!(capacity > 0);
        Arc::new(Port {
            inner: SpinLock::new(Inner {
                queue: VecDeque::with_capacity(capacity),
                capacity,
                senders: Vec::new(),
                receivers: Vec::new(),
            }),
        })
    }

    /// Puts a message on the port, waiting while it is full.
    pub fn send(&self, msg: Message) {
        let me = sched::current_id();
        let mut msg = Some(msg);
        loop {
            {
                let mut p = self.inner.lock();
                if p.queue.len() < p.capacity {
                    p.queue.push_back(msg.take().unwrap());
                    p.senders.retain(|&t| t != me);
                    wake_one(&mut p.receivers);
                    return;
                }
                enlist(&mut p.senders, me);
            }
            sched::park();
        }
    }

    /// Takes the oldest message, waiting while the port is empty.
    pub fn recv(&self) -> Message {
        let me = sched::current_id();
        loop {
            {
                let mut p = self.inner.lock();
                if let Some(m) = p.queue.pop_front() {
                    p.receivers.retain(|&t| t != me);
                    wake_one(&mut p.senders);
                    return m;
                }
                enlist(&mut p.receivers, me);
            }
            sched::park();
        }
    }

    /// Takes a message if one is waiting, without blocking.
    /// Sends without waiting; false (and the message dropped) when the
    /// port is full. Safe from an interrupt handler.
    pub fn try_send(&self, msg: Message) -> bool {
        let mut p = self.inner.lock();
        if p.queue.len() >= p.capacity {
            return false;
        }
        p.queue.push_back(msg);
        wake_one(&mut p.receivers);
        true
    }

    pub fn try_recv(&self) -> Option<Message> {
        let mut p = self.inner.lock();
        let m = p.queue.pop_front();
        if m.is_some() {
            wake_one(&mut p.senders);
        }
        m
    }

    /// Sends a request and waits for the answer on a fresh reply port.
    pub fn call(&self, mut msg: Message) -> Message {
        let reply = Port::new(1);
        msg.reply = Some(reply.clone());
        self.send(msg);
        reply.recv()
    }
}

static NAMES: SpinLock<Vec<(&'static str, Arc<Port>)>> = SpinLock::new(Vec::new());

/// Publishes a port under a name. Returns false if the name is taken.
pub fn publish(name: &'static str, port: Arc<Port>) -> bool {
    let mut names = NAMES.lock();
    if names.iter().any(|(n, _)| *n == name) {
        return false;
    }
    names.push((name, port));
    true
}

/// Finds a published port.
pub fn find(name: &str) -> Option<Arc<Port>> {
    NAMES.lock().iter().find(|(n, _)| *n == name).map(|(_, p)| p.clone())
}

/// Removes a published name.
pub fn withdraw(name: &str) {
    NAMES.lock().retain(|(n, _)| *n != name);
}
