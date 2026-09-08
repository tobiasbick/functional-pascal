use super::*;
use crate::vm::Vm;

fn fixture() -> (Vm, Arc<Lifetime>) {
    let (source, errors) = fpas_parser::parse("program Empty; begin end.");
    assert!(errors.is_empty());
    let vm = Vm::with_writer_and_args(
        fpas_compiler::compile(&source).unwrap(),
        Box::new(std::io::sink()),
        vec![],
    );
    let group = vm
        .scheduler
        .groups
        .create(0, || vm.hosted.cancellations.create_owned())
        .unwrap();
    let life = Arc::new(Lifetime::new(
        group,
        0,
        vm.hosted.cancellations.create_owned(),
        Duration::from_millis(100),
        &vm.scheduler,
        &vm.hosted,
    ));
    (vm, life)
}

#[test]
fn shared_deadline_is_not_extended_by_repeated_stop_requests() {
    let (_vm, life) = fixture();
    assert!(life.request_stop());
    let first = life.progress.lock().unwrap().deadline;
    assert!(!life.request_stop());
    assert_eq!(life.progress.lock().unwrap().deadline, first);
}

#[test]
fn completion_waits_for_explicit_group_close_and_the_owner() {
    let (vm, life) = fixture();
    life.request_stop();
    assert!(life.finish(0).is_err());
    vm.scheduler.groups.begin_close(life.group, 0).unwrap();
    vm.scheduler.groups.take_closed(life.group).unwrap();
    assert!(life.finish(1).is_err());
    assert!(life.finish(0).unwrap());
    assert!(life.finish(0).unwrap());
}

#[test]
fn signal_dispatch_requests_the_same_stop_without_restarting_the_deadline() {
    let (_vm, life) = fixture();
    signals::subscribe_for_test(&life);
    signals::dispatch();
    assert!(!life.ready());
    assert!(life.cancellation.is_cancelled());
    let deadline = life.progress.lock().unwrap().deadline;
    signals::dispatch();
    assert_eq!(life.progress.lock().unwrap().deadline, deadline);
}
