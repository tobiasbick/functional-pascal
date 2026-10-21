//! Real runtime holders keep local cycles across maintenance and release them afterward.
mod automatic;
use super::*;
use crate::vm::{
    HostChild, Vm,
    worker::{FunctionInputs, Worker},
};
use fpas_bytecode::{DomainPhase, FunctionId, MaintenanceOutcome, OwnedCell, RootKind};

fn image(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).unwrap()
}
fn worker() -> Worker {
    Worker::new(Arc::new(image(
        "program Roots; begin const N := 1; discard N; end.",
    )))
    .unwrap()
}
fn cycle(domain: &MemoryDomain) -> OwnedCell {
    let cell = domain.new_cell(Value::Unit).unwrap();
    cell.replace(domain, Value::option_some(Value::Cell(cell.clone())))
        .unwrap();
    cell
}
fn collect(domain: &MemoryDomain) {
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
}
#[test]
fn returned_self_recursive_closure_survives_vm_drop_and_collects_after_export_release() {
    let mut vm = Vm::new(image(
        "program ExportedCycle; function Make(): procedure(); begin var Current: procedure() := procedure() begin end procedure; Current := procedure() begin Current(); end procedure; return Current; end function; begin end.",
    ));
    let domain = vm.domain.clone();
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
    let exported = vm
        .call(FunctionId::try_from_index(function).unwrap(), vec![])
        .unwrap()
        .value;
    let capture = exported.child(HostChild::Capture(0)).unwrap().unwrap();
    let alias = capture.clone();
    let cells: Vec<_> = domain
        .live_cells()
        .iter()
        .map(OwnedCell::downgrade)
        .collect();
    assert!(!cells.is_empty());
    drop(vm);
    assert_eq!(domain.phase(), DomainPhase::Retired);
    collect(&domain);
    assert!(cells.iter().all(|cell| cell.upgrade().is_some()));
    assert_eq!(capture, alias);
    assert_eq!(capture.read_cell().unwrap().type_name(), "function");
    drop(exported);
    drop(capture);
    collect(&domain);
    assert!(cells.iter().all(|cell| cell.upgrade().is_some()));
    drop(alias);
    collect(&domain);
    assert!(cells.iter().all(|cell| cell.upgrade().is_none()));
    assert!(domain.collection_stats().cleared_cells > 0);
}
#[test]
fn saved_completion_roots_survive_migration_and_release_without_a_live_worker() {
    let mut worker = worker();
    let domain = worker.domain.clone();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    worker.pending_completion = Some(Ok(Value::Cell(cell)));
    worker.reset_registers(0);
    let mut saved = worker.take_task_state();
    drop(worker);
    collect(&domain);
    assert!(weak.upgrade().is_some());
    {
        let _access = domain.access();
        saved.pending_completion = None;
        saved.publish_execution();
    }
    collect(&domain);
    assert!(
        weak.upgrade().is_none(),
        "stats: {:?}, roots: {}",
        domain.collection_stats(),
        domain.root_count()
    );
}
#[test]
fn native_caller_roots_preserve_a_cycle_while_nested_execution_collects_garbage() {
    let mut parent = worker();
    let domain = parent.domain.clone();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    parent.store_register(0, Value::Cell(cell)).unwrap();
    let native = parent.native_caller_roots().unwrap();
    parent.reset_registers(0);
    parent.publish_execution(&[]).unwrap();
    let garbage = cycle(&domain);
    let garbage_weak = garbage.downgrade();
    drop(garbage);
    let callback = Worker::for_function_in_domain(
        parent.executable.clone(),
        parent.function,
        FunctionInputs {
            arguments: &[],
            captures: &[],
            domain: domain.clone(),
        },
        parent.globals.clone(),
        parent.layouts.clone(),
        parent.hosted.clone(),
    )
    .unwrap();
    let access = callback.execution_access().unwrap();
    callback.publish_execution(&[]).unwrap();
    assert_eq!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Deferred
    );
    assert!(matches!(
        access.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_some());
    assert!(garbage_weak.upgrade().is_none());
    drop(access);
    drop(callback);
    drop(native);
    parent.publish_execution(&[]).unwrap();
    collect(&domain);
    assert!(
        weak.upgrade().is_none(),
        "stats: {:?}, roots: {}",
        domain.collection_stats(),
        domain.root_count()
    );
}
#[test]
fn closing_an_owner_case_releases_its_cycle_but_preserves_an_independent_debugger_root() {
    use fpas_bytecode::{Intrinsic, TaskIntrinsic};
    let mut worker = worker();
    let domain = worker.domain.clone();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    let function = worker.executable.function_target(worker.function).unwrap();
    let callback = Value::task_owned_function(
        function,
        vec![Value::Cell(cell.clone())],
        domain.id().clone(),
    );
    let inspection = domain.root_values(RootKind::Inspection, vec![Value::Cell(cell)]);
    worker
        .execute_debug_intrinsic(
            Intrinsic::Task(TaskIntrinsic::TimerCase),
            &[Value::Integer(60_000), callback],
        )
        .unwrap();
    worker.reset_registers(0);
    worker.publish_execution(&[]).unwrap();
    collect(&domain);
    assert!(weak.upgrade().is_some());
    worker.retire_execution();
    collect(&domain);
    assert!(weak.upgrade().is_some());
    drop(inspection);
    collect(&domain);
    assert!(
        weak.upgrade().is_none(),
        "stats: {:?}, roots: {}",
        domain.collection_stats(),
        domain.root_count()
    );
}

#[test]
fn a_requesting_thread_collects_a_published_native_wait_without_releasing_its_values() {
    let worker = worker();
    let domain = worker.domain.clone();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    let garbage = cycle(&domain);
    let garbage_weak = garbage.downgrade();
    drop(garbage);
    let access = worker.execution_access().unwrap();
    let wait = worker.native_wait(vec![Value::Cell(cell)]).unwrap();
    std::thread::scope(|scope| {
        scope.spawn(|| collect(&domain)).join().unwrap();
    });
    assert!(weak.upgrade().is_some());
    assert!(garbage_weak.upgrade().is_none());
    drop(wait);
    worker.publish_execution(&[]).unwrap();
    assert_eq!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Deferred
    );
    assert!(matches!(
        access.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_none());
}
