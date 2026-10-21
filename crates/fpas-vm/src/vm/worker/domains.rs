//! Logical-task ownership survives suspension without leaking into another worker.

use super::*;

fn executable() -> Arc<VerifiedExecutable> {
    let (program, errors) =
        fpas_parser::parse("program Domain; begin const Number := 1; discard Number; end.");
    assert!(errors.is_empty(), "{errors:?}");
    Arc::new(fpas_compiler::compile(&program).expect("domain fixture"))
}

#[test]
fn task_migration_preserves_cell_access_and_alias_identity() {
    let mut worker = Worker::new(executable()).expect("worker");
    let original_domain = worker.domain.clone();
    let cell = original_domain.new_cell(Value::Integer(41)).expect("cell");
    worker
        .store_register(0, Value::Cell(cell.clone()))
        .expect("capture register");

    let template = worker.pool_template();
    assert!(cell.read(&template.domain).is_err());
    let saved = worker.take_task_state();
    let resumed = template.worker_for_task(saved);
    let Value::Cell(restored) = &resumed.registers[0] else {
        panic!("saved cell");
    };
    assert!(cell.ptr_eq(restored));
    assert_eq!(
        restored.read(&resumed.domain).expect("owner read"),
        Value::Integer(41)
    );
    restored
        .replace(&resumed.domain, Value::Integer(42))
        .expect("owner write after migration");
    assert_eq!(
        cell.read(&original_domain).expect("original alias"),
        Value::Integer(42)
    );
}

#[test]
fn equal_scheduler_numbers_do_not_grant_cross_session_cell_access() {
    let image = executable();
    let owner = Worker::new(Arc::clone(&image)).expect("owner");
    let foreign = Worker::new(image).expect("foreign session");
    assert_eq!(owner.task_id, foreign.task_id);
    let cell = owner.domain.new_cell(Value::Integer(7)).expect("cell");
    assert!(cell.read(&foreign.domain).is_err());
    assert!(cell.replace(&foreign.domain, Value::Integer(8)).is_err());
    assert_eq!(
        cell.read(&owner.domain).expect("unchanged"),
        Value::Integer(7)
    );
}
