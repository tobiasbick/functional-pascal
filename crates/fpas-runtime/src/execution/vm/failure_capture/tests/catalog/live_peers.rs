//! Running pool peers answer the cooperative cut; the timing isolates collection from cleanup.

use super::*;
use crate::execution::vm::{FailureScalar, hosted::HostedState};
use std::{
    io::{self, Write},
    sync::mpsc,
    time::{Duration, Instant},
};

const SOURCE: &str = "program LivePeers; uses Std.Console as Console;\nprocedure Barrier(); begin end procedure;\nprocedure Running(); begin const Live:=29; Console.WriteText('running'); while true do null; end while; end procedure;\nbegin panic('root'); end.";

struct Ready(mpsc::Sender<()>);
impl Write for Ready {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.send(()).unwrap();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Run `peers` executing pool tasks, then time the root failure until its observation exists.
fn observe_live_peers(image: &Arc<crate::VerifiedExecutable>, peers: usize) -> Duration {
    let mut vm = Vm::from_shared(Arc::clone(image));
    vm.pool_size = 0;
    let output = observe(&mut vm);
    let (ready_tx, ready_rx) = mpsc::channel();
    vm.hosted = Arc::new(HostedState::new(
        crate::stdlib::Console::with_writer(Box::new(Ready(ready_tx))),
        vec![],
    ));
    let ids: Vec<_> = (0..peers)
        .map(|_| enqueue(&vm, "Running", vec![]))
        .collect();
    let pools: Vec<_> = (0..peers).map(|_| start_pool(&vm)).collect();
    for _ in 0..peers {
        ready_rx.recv_timeout(Duration::from_secs(4)).unwrap();
    }
    let started = Instant::now();
    assert_eq!(vm.run().unwrap_err().message, "panic: root");
    let elapsed = started.elapsed();
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    for id in ids {
        let task = catalog.tasks.iter().find(|task| task.id == id).unwrap();
        assert!(matches!(task.state, "running" | "runnable"), "{task:?}");
        assert_eq!(task.inspection, "available", "{task:?}");
        assert_eq!(
            scalar(&task.snapshot.as_ref().unwrap().frames[0], "Live"),
            &Some(FailureScalar::Integer("29".to_string()))
        );
    }
    drop(output);
    drop(pools);
    elapsed
}

#[test]
fn every_running_pool_peer_answers_before_the_collection_deadline() {
    let image = Arc::clone(&vm(SOURCE).executable);
    let elapsed = observe_live_peers(&image, 4);
    assert!(elapsed < Duration::from_millis(500), "{elapsed:?}");
}

/// Local profiling: `cargo test -p fpas-runtime --release live_peer_collection_profile -- --ignored --nocapture`.
#[test]
#[ignore = "local release profiling; prints timings instead of asserting a performance bound"]
fn live_peer_collection_profile() {
    let image = Arc::clone(&vm(SOURCE).executable);
    for peers in [0, 1, 4, 16] {
        observe_live_peers(&image, peers);
        let mut samples: Vec<_> = (0..21).map(|_| observe_live_peers(&image, peers)).collect();
        samples.sort();
        eprintln!(
            "live_peers={peers} min={:?} median={:?} max={:?}",
            samples[0], samples[10], samples[20]
        );
    }
}
