//! Runtime ownership regressions for first-class functions with mutable captures.

use std::sync::Arc;

use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::calls::{FunctionSpec, abc, image};
use super::support::{abx, execute};
use crate::vm::dispatch::DispatchStep;
use crate::vm::worker::Worker;

#[test]
fn mutable_cell_capture_marks_numeric_closure_task_bound() {
    let executable = image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeCell, 0, 0, 0, 0),
            abc(Opcode::MakeClosure, 1, 1, 0, 1),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
        ],
        vec![Constant::Integer(1)],
        &[
            FunctionSpec {
                start: 0,
                end: 4,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 4,
                end: 5,
                arity: 0,
                captures: 1,
                registers: 1,
                returns: ReturnConvention::Unit,
            },
        ],
    );

    let (_, registers, _) = execute(executable).expect("closure construction should succeed");
    let Value::Function(function) = &registers[1] else {
        panic!("register must contain closure")
    };
    assert!(function.task_bound);
    let Value::Cell(cell) = &registers[0] else {
        panic!("capture cell")
    };
    assert_eq!(function.owner_domain.as_ref(), Some(cell.owner()));
    assert_eq!(function.function, FunctionId::new(1));
}

#[test]
fn task_owned_closure_rejects_call_value_from_a_foreign_task() {
    let executable = image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeCell, 0, 0, 0, 0),
            abc(Opcode::MakeClosure, 1, 1, 0, 1),
            abc(Opcode::CallValue, 2, 1, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
        ],
        vec![Constant::Integer(1)],
        &[
            FunctionSpec {
                start: 0,
                end: 5,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 5,
                end: 6,
                arity: 0,
                captures: 1,
                registers: 1,
                returns: ReturnConvention::Unit,
            },
        ],
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    for _ in 0..3 {
        match worker.dispatch_one().expect("setup") {
            DispatchStep::Continue => {}
            DispatchStep::Suspend | DispatchStep::Return(_) => panic!("unexpected setup step"),
        }
    }
    let Value::Function(function) = &worker.registers[1] else {
        panic!("closure")
    };
    assert_eq!(function.owner_domain.as_ref(), Some(worker.domain.id()));

    // A different session may use the same numeric task ID.
    worker.domain = fpas_bytecode::MemoryDomain::new();
    worker.execution = worker.domain.execution_roots().ok();
    assert_eq!(worker.task_id, 0);
    let error = worker
        .dispatch_one()
        .err()
        .expect("a foreign task must not invoke the closure");

    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert!(error.message.contains("foreign task"), "{}", error.message);
}

fn ownership_worker() -> Worker {
    Worker::new(Arc::new(super::support::verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "ownership.fpas"],
        4,
    )))
    .expect("worker")
}

#[test]
fn nested_same_domain_capture_is_local_and_foreign_capture_is_rejected() {
    let mut worker = ownership_worker();
    let cell = worker.domain.new_cell(Value::Integer(7)).expect("cell");
    worker.registers[0] = Value::option_some(Value::Array(vec![Value::Cell(cell.clone())].into()));
    worker
        .make_closure(abc(Opcode::MakeClosure, 1, 0, 0, 1).abc_payload())
        .expect("same-domain nested capture");
    let Value::Function(function) = &worker.registers[1] else {
        panic!("closure")
    };
    assert!(function.task_bound);
    assert_eq!(function.owner_domain.as_ref(), Some(worker.domain.id()));

    let foreign = fpas_bytecode::MemoryDomain::new();
    worker.registers[0] = Value::option_some(Value::Array(
        vec![Value::Cell(
            foreign.new_cell(Value::Integer(8)).expect("foreign cell"),
        )]
        .into(),
    ));
    let previous = worker.registers[1].clone();
    let error = worker
        .make_closure(abc(Opcode::MakeClosure, 1, 0, 0, 1).abc_payload())
        .expect_err("foreign nested capture");
    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert_eq!(
        worker.registers[1], previous,
        "failed construction preserves the destination"
    );
    assert_eq!(
        cell.read(&worker.domain).expect("original cell"),
        Value::Integer(7)
    );
}

#[test]
fn unbound_host_callable_cannot_hide_foreign_cells_or_task_bound_captures() {
    let worker = ownership_worker();
    let foreign = fpas_bytecode::MemoryDomain::new();
    let target = worker.function_target(FunctionId::new(0)).expect("target");
    for hidden in [
        Value::option_some(Value::Cell(
            foreign.new_cell(Value::Integer(1)).expect("cell"),
        )),
        Value::task_owned_function(target.clone(), Vec::new(), foreign.id().clone()),
    ] {
        let function = fpas_bytecode::SharedFunction::unbound(target.clone(), vec![hidden]);
        assert_eq!(
            worker
                .require_function_task_owner(&function)
                .expect_err("foreign graph")
                .code,
            RUNTIME_INVALID_TASK
        );
    }
}

#[test]
fn nested_helpers_can_borrow_var_parameters_through_direct_and_transitive_calls() {
    let (program, errors) = fpas_parser::parse(
        "program BorrowedCaptures;
         var Observed: integer := 0;
         procedure Outer(var Total: integer);
           procedure First(); begin Total := Total + 1; end procedure;
           procedure Second(); begin First(); end procedure;
         begin Second(); end procedure;
         begin
           var Local := 4;
           Outer(var Local);
           Observed := Local;
           Outer(var Observed);
         end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("borrowed helper program");
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    loop {
        match worker.dispatch_one().expect("borrowed helper call") {
            DispatchStep::Continue => {}
            DispatchStep::Return(_) => break,
            DispatchStep::Suspend => panic!("synchronous helper must not suspend"),
        }
    }
    assert_eq!(worker.global_slots()[0], Some(Value::Integer(6)));
}

#[test]
fn borrowed_helper_captures_remain_local_and_cannot_enter_persistent_storage() {
    use fpas_bytecode::{OwnershipError, ReferenceRoot, VariableReference};

    let mut worker = ownership_worker();
    let cell = worker.domain.new_cell(Value::Integer(7)).expect("referent");
    for root in [
        ReferenceRoot::Cell(cell.clone()),
        ReferenceRoot::Global {
            slot: 0,
            owner: worker.domain.id().clone(),
        },
    ] {
        worker.registers[0] = Value::Reference(Arc::new(VariableReference {
            root,
            path: Vec::new(),
        }));
        worker
            .make_closure(abc(Opcode::MakeClosure, 1, 0, 0, 1).abc_payload())
            .expect("local ephemeral helper");
        let value = worker.registers[1].clone();
        let Value::Function(function) = &value else {
            panic!("helper function expected");
        };
        assert!(function.task_bound);
        worker
            .require_function_task_owner(function)
            .expect("local invocation may borrow a reference");
        assert!(
            !fpas_bytecode::classify_values(std::slice::from_ref(&value))
                .expect("helper classification")
                .is_transferable()
        );
        assert_eq!(
            cell.replace(&worker.domain, value.clone()),
            Err(OwnershipError::EscapingReference)
        );
        assert_eq!(
            cell.read(&worker.domain).expect("unchanged"),
            Value::Integer(7)
        );
        assert!(worker.require_storable_value(&value).is_err());

        let mut foreign = ownership_worker();
        assert_eq!(
            foreign
                .require_function_task_owner(function)
                .expect_err("foreign invocation")
                .code,
            RUNTIME_INVALID_TASK
        );
        foreign.registers[0] = worker.registers[0].clone();
        assert_eq!(
            foreign
                .make_closure(abc(Opcode::MakeClosure, 1, 0, 0, 1).abc_payload())
                .expect_err("foreign reference capture")
                .code,
            RUNTIME_INVALID_TASK
        );
        assert_eq!(foreign.registers[1], Value::Unit);
    }
}

#[test]
fn cell_reads_and_writes_reject_foreign_domains_without_changing_storage() {
    let mut worker = ownership_worker();
    let original = worker.domain.new_cell(Value::Integer(11)).expect("cell");
    let foreign = fpas_bytecode::MemoryDomain::new();
    let foreign_cell = foreign.new_cell(Value::Integer(23)).expect("foreign cell");
    worker.registers[0] = Value::Cell(original.clone());
    worker.registers[1] =
        Value::result_ok(Value::Array(vec![Value::Cell(foreign_cell.clone())].into()));
    assert_eq!(
        worker
            .write_cell(abc(Opcode::CellWrite, 0, 1, 0, 0).abc_payload())
            .expect_err("foreign payload")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(
        original.read(&worker.domain).expect("unchanged"),
        Value::Integer(11)
    );

    worker.registers[1] = Value::Cell(foreign_cell.clone());
    worker.registers[2] = Value::Integer(41);
    assert_eq!(
        worker
            .read_cell(abc(Opcode::CellRead, 2, 1, 0, 0).abc_payload())
            .expect_err("foreign read")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(worker.registers[2], Value::Integer(41));
    assert_eq!(
        worker
            .write_cell(abc(Opcode::CellWrite, 1, 2, 0, 0).abc_payload())
            .expect_err("foreign destination")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(
        foreign_cell.read(&foreign).expect("foreign untouched"),
        Value::Integer(23)
    );
}

#[test]
fn var_cell_paths_validate_replacements_before_commit() {
    use fpas_bytecode::{ReferenceRoot, ReferenceStep, VariableReference};
    let mut worker = ownership_worker();
    let cell = worker
        .domain
        .new_cell(Value::Array(
            vec![Value::Integer(5), Value::Integer(6)].into(),
        ))
        .expect("cell");
    let reference = VariableReference {
        root: ReferenceRoot::Cell(cell.clone()),
        path: vec![ReferenceStep::Element(1)],
    };
    worker
        .write_reference(&reference, Value::Integer(9))
        .expect("valid path update");
    assert_eq!(
        worker.read_reference(&reference).expect("aliased read"),
        Value::Integer(9)
    );

    let foreign = fpas_bytecode::MemoryDomain::new();
    let incoming = Value::option_some(Value::Cell(
        foreign.new_cell(Value::Unit).expect("foreign cell"),
    ));
    assert_eq!(
        worker
            .write_reference(&reference, incoming)
            .expect_err("foreign replacement")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(
        worker.read_reference(&reference).expect("unchanged path"),
        Value::Integer(9)
    );
    let invalid_path = reference.narrowed(ReferenceStep::Field(0));
    assert!(
        worker
            .write_reference(&invalid_path, Value::Integer(15))
            .is_err()
    );
    assert_eq!(
        worker
            .read_reference(&reference)
            .expect("unchanged invalid path"),
        Value::Integer(9)
    );
}

#[test]
fn numeric_global_references_cannot_cross_domain_identity() {
    use fpas_bytecode::{ReferenceRoot, VariableReference};
    let mut worker = ownership_worker();
    let foreign = fpas_bytecode::MemoryDomain::new();
    let reference = VariableReference {
        root: ReferenceRoot::Global {
            slot: 0,
            owner: foreign.id().clone(),
        },
        path: Vec::new(),
    };
    assert_eq!(
        worker
            .read_reference(&reference)
            .expect_err("foreign global read")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(
        worker
            .write_reference(&reference, Value::Integer(1))
            .expect_err("foreign global write")
            .code,
        RUNTIME_INVALID_TASK
    );
}

#[test]
fn array_mutation_rejects_foreign_payload_before_consuming_destination() {
    let mut worker = ownership_worker();
    let foreign = fpas_bytecode::MemoryDomain::new();
    worker.registers[0] = Value::Array(vec![Value::Integer(4)].into());
    worker.registers[1] = Value::Integer(0);
    worker.registers[2] = Value::Cell(foreign.new_cell(Value::Unit).expect("foreign cell"));
    let previous = worker.registers[0].clone();
    assert_eq!(
        worker
            .index_set(abc(Opcode::IndexSet, 0, 1, 2, 0).abc_payload())
            .expect_err("foreign array element")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(worker.registers[0], previous);
    assert_eq!(
        worker
            .array_push(abc(Opcode::ArrayPush, 0, 0, 2, 0).abc_payload())
            .expect_err("foreign appended element")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(worker.registers[0], previous);
}
