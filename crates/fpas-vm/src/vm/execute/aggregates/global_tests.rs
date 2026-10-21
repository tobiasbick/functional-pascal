//! Immutable global storage does not authorize access to another task's local graph.

use std::sync::Arc;

use fpas_bytecode::{Instruction, MemoryDomain, Opcode, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::vm::worker::Worker;

fn worker(local_globals: bool) -> Worker {
    let (program, errors) = fpas_parser::parse(
        "program GlobalOwnership; const Stored: array of integer := [1]; begin discard Stored.Length(); end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let executable = Arc::new(fpas_compiler::compile(&program).expect("global fixture"));
    let mut worker = if local_globals {
        Worker::new_with_local_globals(executable)
    } else {
        Worker::new(executable)
    }
    .expect("worker");
    assert_eq!(worker.global_slots().len(), 1);
    assert!(!worker.executable.executable().globals[0].mutable);
    worker.registers[0] = Value::Integer(99);
    worker.register_initialized[0] = true;
    worker
}

fn load(worker: &mut Worker) -> Result<(), crate::vm::VmError> {
    worker.load_global(
        Instruction::abx(Opcode::LoadGlobal, 0, 0)
            .expect("load global")
            .abx_payload(),
    )
}

#[test]
fn immutable_global_local_graph_is_accessible_only_in_its_own_domain() {
    for local_globals in [false, true] {
        for wrapped in [false, true] {
            let mut worker = worker(local_globals);
            let target = worker
                .executable
                .function_target(worker.executable.executable().entry)
                .expect("target");
            let local = Value::task_owned_function(target, Vec::new(), worker.domain.id().clone());
            let stored = if wrapped {
                Value::option_some(Value::Array(vec![local].into()))
            } else {
                local
            };
            // Install below initialization to exercise the load boundary independently of sema.
            worker.global_slots_mut()[0] = Some(stored.clone());
            load(&mut worker).expect("owner may use local global contents");
            assert_eq!(worker.registers[0], stored);
            worker.registers[0] = Value::Integer(99);
            worker.domain = MemoryDomain::new();
            worker.task_id = 1;
            let error = load(&mut worker).expect_err("foreign local graph");
            assert_eq!(error.code, RUNTIME_INVALID_TASK);
            assert_eq!(worker.registers[0], Value::Integer(99));
            assert_eq!(worker.global_slots()[0], Some(stored));
        }
    }
}

#[test]
fn immutable_global_nested_cells_cannot_bypass_callable_flags() {
    let mut worker = worker(false);
    let cell = worker.domain.new_cell(Value::Integer(4)).expect("cell");
    worker.global_slots_mut()[0] = Some(Value::option_some(Value::Array(
        vec![Value::Cell(cell)].into(),
    )));
    worker.domain = MemoryDomain::new();
    worker.task_id = 1;
    assert_eq!(
        load(&mut worker).expect_err("foreign cell").code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(worker.registers[0], Value::Integer(99));
}

#[test]
fn ordinary_immutable_global_data_remains_available_to_workers() {
    for local_globals in [false, true] {
        let mut worker = worker(local_globals);
        let stored = Value::option_some(Value::Array(vec![Value::Integer(17)].into()));
        worker.global_slots_mut()[0] = Some(stored.clone());
        worker.domain = MemoryDomain::new();
        worker.task_id = 1;
        load(&mut worker).expect("transferable global data");
        assert_eq!(worker.registers[0], stored);
    }
}
