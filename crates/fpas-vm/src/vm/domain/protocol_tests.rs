//! Runtime boundaries exercise the permit route without enabling reclamation.
use crate::vm::{Vm, worker::Worker};
use fpas_bytecode::{
    DomainMaintenance, MaintenanceOutcome, MaintenancePolicy, MemoryDomain, OwnershipError, Value,
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

struct Probe(Mutex<mpsc::Sender<(bool, Vec<u64>)>>);
impl DomainMaintenance for Probe {
    fn maintain(&self, _: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        let local = !fpas_bytecode::classify_values(roots)?.owners.is_empty();
        let handles = roots
            .iter()
            .filter_map(|value| match value {
                Value::OpaqueHandle(handle) => Some(*handle),
                _ => None,
            })
            .collect();
        self.0.lock().unwrap().send((local, handles)).unwrap();
        Ok(())
    }
}
struct BarrierWriter {
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    first: bool,
}
impl Write for BarrierWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.first {
            self.first = false;
            self.entered.send(()).unwrap();
            self.release.recv().unwrap();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Stop(Arc<crate::vm::hosted::HostedState>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.shutdown_blocking_operations();
    }
}
struct Release(mpsc::Sender<()>);
impl Drop for Release {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}
fn image(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("fixture")
}

#[test]
fn main_channel_wait_services_requests_before_its_peer_sends() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (pass_tx, pass_rx) = mpsc::channel();
    let mut vm = Vm::with_writer(
        image(
            r#"program PublishedChannel;
uses Std.Tasks as Tasks, Std.Console as Console;
var C: channel of integer := Tasks.CreateChannel(1);
begin
  Console.WriteText('ready');
  if Tasks.Receive(C).Unwrap() <> 7 then panic('wrong peer'); end if;
end."#,
        ),
        Box::new(BarrierWriter {
            entered: entered_tx,
            release: release_rx,
            first: true,
        }),
    );
    let domain = vm.domain.clone();
    let hosted = vm.hosted.clone();
    domain.set_maintenance(
        Arc::new(Probe(Mutex::new(pass_tx))),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let vm_hosted = hosted.clone();
        let running = scope.spawn(move || vm.run());
        let _stop = Stop(vm_hosted.clone());
        let release = Release(release_tx);
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Deferred
        );
        assert!(
            pass_rx.try_recv().is_err(),
            "opaque output keeps its execution permit"
        );
        drop(release);
        let (_, handles) = pass_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("published channel wait services request");
        assert!(
            !running.is_finished(),
            "maintenance does not execute or complete the receive"
        );
        assert!(matches!(
            hosted
                .channels
                .send(
                    *handles.first().expect("published channel identity"),
                    Value::Integer(7),
                    false,
                    None
                )
                .unwrap(),
            crate::vm::channels::SendState::Sent
        ));
        running.join().unwrap().unwrap();
    });
}

#[test]
fn opaque_native_return_services_the_pending_request_before_export() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (pass_tx, pass_rx) = mpsc::channel();
    let mut vm = Vm::with_writer(
        image(
            r#"program OpaqueReturn;
uses Std.Console as Console;
function Make(): function(): integer;
begin
  var N := 7;
  Console.WriteText('ready');
  return function(): integer begin N := N + 1; return N; end function;
end function;
begin end."#,
        ),
        Box::new(BarrierWriter {
            entered: entered_tx,
            release: release_rx,
            first: true,
        }),
    );
    let function = vm
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| {
            vm.executable
                .executable()
                .strings
                .get(info.name)
                .is_some_and(|name| name.eq_ignore_ascii_case("Make"))
        })
        .unwrap();
    let domain = vm.domain.clone();
    domain.set_maintenance(
        Arc::new(Probe(Mutex::new(pass_tx))),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let running = scope.spawn(move || {
            vm.call(
                fpas_bytecode::FunctionId::try_from_index(function).unwrap(),
                vec![],
            )
        });
        let release = Release(release_tx);
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Deferred
        );
        assert!(pass_rx.try_recv().is_err());
        drop(release);
        assert!(
            pass_rx.recv_timeout(Duration::from_secs(10)).unwrap().0,
            "final callable capture is a prepared root"
        );
        let execution = running.join().unwrap().unwrap();
        assert_eq!(domain.phase(), fpas_bytecode::DomainPhase::Retired);
        assert!(
            !domain.live_cells().is_empty(),
            "export survives worker and VM destruction"
        );
        drop(execution);
        assert!(domain.live_cells().is_empty());
    });
}

#[test]
fn saved_worker_waits_for_collection_and_is_claimed_once() {
    let mut worker = Worker::new(Arc::new(image(
        "program Saved; begin const N := 1; discard N; end.",
    )))
    .unwrap();
    let cell = worker.domain.new_cell(Value::Integer(12)).unwrap();
    worker.store_register(0, Value::Cell(cell)).unwrap();
    let domain = worker.domain.clone();
    let template = worker.pool_template();
    let saved = worker.take_task_state();
    let scheduler = Arc::new(crate::vm::tasks::TaskScheduler::new());
    scheduler.enqueue(saved);
    struct Blocking {
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl DomainMaintenance for Blocking {
        fn maintain(&self, _: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
            assert!(roots.iter().any(|value| matches!(value, Value::Cell(_))));
            self.entered.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            Ok(())
        }
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (claimed_tx, claimed_rx) = mpsc::channel();
    let (resumed_tx, resumed_rx) = mpsc::channel();
    domain.set_maintenance(
        Arc::new(Blocking {
            entered: entered_tx,
            release: Mutex::new(release_rx),
        }),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let collect = scope.spawn(|| domain.request_explicit_maintenance());
        let release = Release(release_tx);
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let scheduler2 = scheduler.clone();
        let resume = scope.spawn(move || {
            let state = scheduler2.try_dequeue().expect("claim saved task");
            claimed_tx.send(()).unwrap();
            let resumed = template.worker_for_task(state);
            let _access = resumed.execution_access().unwrap();
            resumed_tx.send(()).unwrap();
            assert!(matches!(&resumed.registers[0], Value::Cell(_)));
        });
        claimed_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(resumed_rx.try_recv().is_err());
        assert!(scheduler.try_dequeue().is_none());
        drop(release);
        assert!(matches!(
            collect.join().unwrap(),
            MaintenanceOutcome::Serviced(_)
        ));
        resumed_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        resume.join().unwrap();
    });
}

#[test]
fn nested_callback_boundaries_publish_enclosing_native_roots() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (pass_tx, pass_rx) = mpsc::channel();
    let mut vm = Vm::with_writer(
        image(
            r#"program NestedRoots;
uses Std.Console as Console;
function Outer(): integer;
begin
  var Total := 0;
  discard [1].Map(function(X: integer): integer begin
    const Inner := [2].Map(function(Y: integer): integer begin
      Console.WriteText('ready');
      Total := Total + X + Y;
      return Total;
    end function);
    return Inner[0];
  end function);
  return Total;
end function;
begin if Outer() <> 3 then panic('callback result'); end if; end."#,
        ),
        Box::new(BarrierWriter {
            entered: entered_tx,
            release: release_rx,
            first: true,
        }),
    );
    let domain = vm.domain.clone();
    domain.set_maintenance(
        Arc::new(Probe(Mutex::new(pass_tx))),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let running = scope.spawn(move || vm.run());
        let release = Release(release_tx);
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Deferred
        );
        drop(release);
        assert!(pass_rx.recv_timeout(Duration::from_secs(10)).unwrap().0);
        running.join().unwrap().unwrap();
    });
}
