//! Large task sets preserve preferred identities and success releases all catalog charges.

use super::*;
use crate::execution::vm::FailureScalar;
use std::time::Instant;

#[test]
fn task_limit_keeps_main_and_a_cause_outside_the_tracked_prefix() {
    let mut vm = vm(
        "program Bounds; procedure Idle(); begin null; end procedure; procedure Fail(Number:integer); begin panic('selected'); end procedure; begin end.",
    );
    let output = observe(&mut vm);
    for _ in 0..1200 {
        enqueue(&vm, "Idle", vec![]);
    }
    let mut main = Worker::new(Arc::clone(&vm.executable))
        .unwrap()
        .with_scheduler(Some(Arc::clone(&vm.scheduler)));
    let capture = Arc::clone(vm.scheduler.failure_capture.get().unwrap());
    capture.start_root(&mut main);
    let mut cause = worker(&vm, "Fail", vec![Value::Integer(73)]);
    cause.task_id = vm.scheduler.alloc_id();
    let id = cause.task_id;
    vm.scheduler.register_result(id);
    let error = cause.run_task().unwrap_err();
    vm.scheduler.store_failure(id, error.clone());
    vm.notify_failure(&error);
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    assert_eq!(catalog.total_tasks, 1202);
    assert!(catalog.tasks.len() <= 1024);
    assert_eq!(
        catalog.omitted_tasks,
        catalog.total_tasks - catalog.tasks.len()
    );
    assert!(catalog.omitted_tasks > 0);
    assert!(catalog.tasks.iter().any(|task| task.id == 0));
    let selected = catalog.tasks.iter().find(|task| task.id == id).unwrap();
    assert_eq!(selected.observation, "fault");
    assert_eq!(
        scalar(&selected.snapshot.as_ref().unwrap().frames[0], "Number"),
        &Some(FailureScalar::Integer("73".into()))
    );
    assert!(catalog.tasks.windows(2).all(|pair| pair[0].id < pair[1].id));
    drop(output);
    capture.finish_catalog();
}

#[test]
fn successful_runs_release_catalog_storage_and_disabled_runs_do_not_install_it() {
    let source = "program Success; begin var Total:=0; for I:integer:=1 to 20000 do Total:=Total+I; end for; if Total<>200010000 then panic('wrong total'); end if; end.";
    let image = Arc::clone(&vm(source).executable);
    let mut elapsed = [0.0; 2];
    for (mode, enabled) in [false, true].into_iter().enumerate() {
        let start = Instant::now();
        for _ in 0..32 {
            let mut vm = Vm::from_shared(Arc::clone(&image));
            if enabled {
                assert!(vm.set_failure_observer(|_| panic!("successful runs must not notify")));
            }
            vm.run().unwrap();
            if enabled {
                let capture = vm.scheduler.failure_capture.get().unwrap();
                let usage = capture.usage.lock().unwrap();
                assert_eq!((usage.values, usage.bytes), (0, 0));
            } else {
                assert!(vm.scheduler.failure_capture.get().is_none());
            }
        }
        elapsed[mode] = start.elapsed().as_secs_f64() * 1000.0;
    }
    eprintln!(
        "capture success smoke timings: disabled={:.2} ms enabled={:.2} ms ratio={:.3}",
        elapsed[0],
        elapsed[1],
        elapsed[1] / elapsed[0]
    );
}
