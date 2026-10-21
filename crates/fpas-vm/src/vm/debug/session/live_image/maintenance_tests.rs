//! Replacement graph access serializes with exclusive maintenance.
use super::*;
use fpas_bytecode::{FunctionId, Value};

#[test]
fn replacement_graph_validation_waits_for_exclusive_domain_maintenance() {
    use fpas_bytecode::{DomainMaintenance, MaintenancePolicy, MemoryDomain, OwnershipError};
    use std::sync::{Mutex, mpsc};
    use std::time::Duration;
    struct Paused {
        entered: mpsc::Sender<()>,
        leave: Mutex<mpsc::Receiver<()>>,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            self.entered.send(()).unwrap();
            self.leave.lock().unwrap().recv().unwrap();
            Ok(())
        }
    }
    fn fixture(n: i64) -> VerifiedExecutable {
        let source = format!(
            "program Image; function Helper(): integer; begin return {n}; end function; begin discard Helper(); end."
        );
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty());
        fpas_compiler::compile(&program).unwrap()
    }
    let current = fixture(1);
    let candidate = fixture(2);
    let mut session = DebugSession::new(current).unwrap();
    let worker = session.runtime.worker_mut(0).unwrap();
    let domain = worker.domain.clone();
    let function = worker
        .executable
        .function_target(FunctionId::new(1))
        .unwrap();
    worker
        .store_register(0, Value::function(function, vec![]))
        .unwrap();
    worker.publish_execution(&[]).unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (leave_tx, leave_rx) = mpsc::channel();
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    domain.set_maintenance(
        Arc::new(Paused {
            entered: entered_tx,
            leave: Mutex::new(leave_rx),
        }),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let collector = scope.spawn(|| domain.request_explicit_maintenance());
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let replace = scope.spawn(move || {
            started_tx.send(()).unwrap();
            let result = session.replace_live_image(&candidate);
            done_tx.send(()).unwrap();
            result
        });
        started_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(
            done_rx.try_recv().is_err(),
            "validation cannot traverse retained callables during maintenance"
        );
        leave_tx.send(()).unwrap();
        collector.join().unwrap();
        assert!(
            replace.join().unwrap().is_err(),
            "retained callable rejects replacement after exclusive maintenance ends"
        );
    });
}
