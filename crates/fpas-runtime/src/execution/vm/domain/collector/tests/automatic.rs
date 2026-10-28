//! Automatic acceptance through real dispatch, roots and inherited task domains.
use super::*;
use crate::CollectionSchedule;
use crate::execution::vm::{dispatch::DispatchStep, tasks::TaskScheduler};
use std::num::NonZeroUsize;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::time::Duration;

fn schedule(allocations: usize, mutations: usize, batches: usize) -> CollectionSchedule {
    CollectionSchedule {
        allocations: NonZeroUsize::new(allocations).unwrap(),
        mutations: NonZeroUsize::new(mutations).unwrap(),
        safe_points: NonZeroUsize::new(batches).unwrap(),
    }
}
#[test]
fn dispatch_reclaims_repeated_cycles_while_preserving_live_closure_aliases() {
    let executable = Arc::new(image(
        r#"
program Cycles;
function Make(): function(): integer;
begin
  var Count := 0;
  var Current: function(): integer := function(): integer begin return 0; end function;
  Current := function(): integer begin
    Count := Count + 1;
    if Count < 0 then return Current(); end if;
    return Count;
  end function;
  return Current;
end function;
begin
  const Keep := Make();
  const Alias := Keep;
  for I: integer := 1 to 600 do
    const Garbage := Make();
    if Garbage() <> 1 then panic('garbage state'); end if;
    if Keep() <> I * 2 - 1 then panic('live state'); end if;
    if Alias() <> I * 2 then panic('alias state'); end if;
  end for;
end.
"#,
    ));
    for (config, bound) in [
        (CollectionSchedule::default(), 1024),
        (schedule(8, 16, 4), 64),
    ] {
        let mut worker = Worker::new(executable.clone()).unwrap();
        let domain = worker.domain.clone();
        configure(&domain, config);
        let mut maximum_live = 0;
        let mut reclaimed_during_execution = false;
        loop {
            let batch = worker.dispatch_batch::<false>(32).unwrap();
            let stats = domain.collection_stats();
            maximum_live = maximum_live.max(stats.live_cells);
            match batch.step {
                DispatchStep::Continue => reclaimed_during_execution |= stats.cleared_cells > 100,
                DispatchStep::Return(_) => break,
                DispatchStep::Suspend => panic!("no suspension"),
            }
        }
        assert!(reclaimed_during_execution);
        assert!(
            maximum_live < bound,
            "live cell envelope: {maximum_live}, bound: {bound}"
        );
        assert!(domain.collection_stats().allocated_cells > 1200);
        worker.retire_execution();
        assert_eq!(domain.collection_stats().live_cells, 0);
    }
}
#[test]
fn child_domains_and_pool_templates_inherit_overrides_without_sharing_identity() {
    let config = schedule(3, 7, 11);
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker=Worker::new(Arc::new(image(
        "program Spawn; procedure Child(); begin end procedure; begin scope discard go Child(); end scope; end."
    ))).unwrap().with_scheduler(Some(scheduler.clone()));
    configure(&worker.domain, config);
    let saved = loop {
        worker.dispatch_batch::<false>(1).unwrap();
        if let Some(saved) = scheduler.try_dequeue() {
            break saved;
        }
    };
    assert_eq!(saved.domain.collection_schedule(), Some(config));
    assert_ne!(saved.domain.id(), worker.domain.id());
    let template = worker.pool_template();
    assert_eq!(template.domain.collection_schedule(), Some(config));
    let child = template.worker_for_task(saved);
    assert_eq!(child.domain.collection_schedule(), Some(config));
}
#[test]
fn a_published_native_wait_collects_without_ending_the_wait() {
    let mut worker = worker();
    let domain = worker.domain.clone();
    configure(&domain, schedule(usize::MAX, usize::MAX, usize::MAX));
    let access = worker.execution_access().unwrap();
    let garbage = cycle(&domain);
    let dead = garbage.downgrade();
    drop(garbage);
    let live = cycle(&domain);
    let held = live.downgrade();
    worker.reset_registers(0);
    let wait = worker.native_wait(vec![Value::Cell(live)]).unwrap();
    assert!(dead.upgrade().is_none());
    assert!(held.upgrade().is_some());
    drop(wait);
    worker.publish_execution(&[]).unwrap();
    access.safe_point();
    assert!(held.upgrade().is_none());
}
#[test]
fn opaque_execution_defers_collection_until_a_complete_return_boundary() {
    let worker = worker();
    let domain = worker.domain.clone();
    configure(&domain, schedule(1, 1, 1));
    let access = worker.execution_access().unwrap();
    let garbage = cycle(&domain);
    let weak = garbage.downgrade();
    drop(garbage);
    assert!(weak.upgrade().is_some());
    assert!(domain.maintenance_pending());
    worker.publish_execution(&[]).unwrap();
    access.safe_point();
    assert!(weak.upgrade().is_none());
}
#[test]
fn root_release_during_collection_is_serviced_and_an_independent_vm_progresses() {
    struct Paused {
        entered: mpsc::Sender<()>,
        leave: Mutex<mpsc::Receiver<()>>,
        calls: AtomicUsize,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                self.entered.send(()).unwrap();
                self.leave.lock().unwrap().recv().unwrap();
            }
            LocalCollector.maintain(domain, roots)
        }
    }
    let domain = new_domain();
    let guard = domain.access();
    let a = cycle(&domain);
    let dead = a.downgrade();
    let b = cycle(&domain);
    let live = b.downgrade();
    let a = domain.root_values(RootKind::Host, vec![Value::Cell(a)]);
    let b = domain.root_values(RootKind::Host, vec![Value::Cell(b)]);
    drop(guard);
    let (entered, ready) = mpsc::channel();
    let (leave, leaving) = mpsc::channel();
    let hook = Arc::new(Paused {
        entered,
        leave: Mutex::new(leaving),
        calls: AtomicUsize::new(0),
    });
    domain.set_maintenance(
        hook.clone(),
        MaintenancePolicy::Scheduled(CollectionSchedule::default()),
    );
    std::thread::scope(|scope| {
        let collector = scope.spawn(|| domain.request_explicit_maintenance());
        ready.recv_timeout(Duration::from_secs(10)).unwrap();
        drop(a);
        let (done, finished) = mpsc::channel();
        let independent = scope.spawn(move || {
            Vm::new(image(
                "program Independent; begin if 6 * 7 <> 42 then panic('math'); end if; end.",
            ))
            .run()
            .unwrap();
            done.send(()).unwrap();
        });
        let progress = finished.recv_timeout(Duration::from_secs(10));
        leave.send(()).unwrap();
        assert!(
            progress.is_ok(),
            "independent execution progresses during collection"
        );
        independent.join().unwrap();
        assert!(matches!(
            collector.join().unwrap(),
            MaintenanceOutcome::Serviced(_)
        ));
    });
    assert!(dead.upgrade().is_none());
    assert!(live.upgrade().is_some());
    assert!(hook.calls.load(Ordering::SeqCst) >= 2);
    drop(b);
    assert!(live.upgrade().is_none());
}
