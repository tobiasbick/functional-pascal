//! A busy terminal neither holds the protocol pump nor prevents native cancellation.

use super::*;
use std::{
    sync::{atomic::AtomicBool, mpsc},
    time::Duration,
};

#[test]
fn busy_terminal_wait_cancels_before_its_lock_is_released_and_pump_skips_it() {
    let terminals = Terminals::new(Arc::new(PipedProcesses::default()), Limits::default());
    let reserved = terminals.reserve("session", None).unwrap();
    let held = reserved.entry.lock().unwrap();
    terminals.pump().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let id = reserved.id.clone();
    let native = terminals.clone();
    let (entered, entry) = mpsc::channel();
    let (sent, result) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let first = AtomicBool::new(true);
        let outcome = native.status(&id, "session", || {
            if first.swap(false, Ordering::AcqRel) {
                entered.send(()).unwrap();
            }
            stopped.load(Ordering::Acquire)
        });
        sent.send(outcome).unwrap();
    });
    entry.recv_timeout(Duration::from_secs(5)).unwrap();
    stop.store(true, Ordering::Release);
    let outcome = result.recv_timeout(Duration::from_secs(5));
    drop(held);
    thread.join().unwrap();
    assert_eq!(outcome.unwrap(), Err(Fault::Cancelled));
    drop(reserved);
    assert!(!terminals.active());
}
