use std::sync::Arc;

use fpas_bytecode::{Constant, Opcode, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN,
};

use super::*;
use crate::vm::register::RegisterVm;

#[test]
fn scalar_failures_keep_codes_and_sparse_source_locations() {
    for (opcode, code) in [
        (Opcode::DivideInteger, RUNTIME_DIVISION_BY_ZERO),
        (Opcode::RemainderInteger, RUNTIME_MODULO_BY_ZERO),
    ] {
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                abc(opcode, 2, 0, 1),
                return_unit(),
            ],
            vec![Constant::Integer(7), Constant::Integer(0)],
            vec!["root", "test.fpas"],
            3,
        ))
        .expect_err("zero divisor must fail");
        assert_eq!(error.code, code);
        assert_eq!(error.span.line(), 41);
        assert_eq!(error.span.column(), 7);
    }
}

#[test]
fn dynamic_type_mismatch_is_a_runtime_diagnostic() {
    let error = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::AddDynamic, 2, 0, 1),
            return_unit(),
        ],
        vec![
            Constant::String(fpas_bytecode::StringId::new(2)),
            Constant::Integer(1),
        ],
        vec!["root", "test.fpas", "not numeric"],
        3,
    ))
    .expect_err("non-numeric dynamic operand must fail");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn integer_domain_edges_fail_without_panicking() {
    for opcode in [
        Opcode::DivideInteger,
        Opcode::RemainderInteger,
        Opcode::NegateInteger,
        Opcode::NegateDynamic,
    ] {
        let operation = if matches!(opcode, Opcode::NegateInteger | Opcode::NegateDynamic) {
            abc(opcode, 2, 0, 0)
        } else {
            abc(opcode, 2, 0, 1)
        };
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                operation,
                return_unit(),
            ],
            vec![Constant::Integer(i64::MIN), Constant::Integer(-1)],
            vec!["root", "test.fpas"],
            3,
        ))
        .expect_err("minimum-integer domain edge must fail");
        assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }
}

#[test]
fn out_of_range_shift_is_a_numeric_domain_error() {
    let error = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::ShiftLeftInteger, 2, 0, 1),
            return_unit(),
        ],
        vec![Constant::Integer(1), Constant::Integer(64)],
        vec!["root", "test.fpas"],
        3,
    ))
    .expect_err("shift amount 64 must fail");
    assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn main_task_yield_executes_without_a_pool() {
    let (value, _, count) = execute(verified(
        vec![abc(Opcode::Yield, 0, 0, 0), return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    ))
    .expect("main-task yield must be executable in P7");
    assert_eq!(value, Value::Unit);
    assert_eq!(count, 2);
}

#[test]
fn shutdown_handle_cancels_register_execution_before_dispatch() {
    let executable = verified(
        vec![return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut vm = super::super::RegisterVm::new(executable);
    vm.shutdown_handle().shutdown();
    let error = vm
        .run()
        .expect_err("pre-run cancellation must stop dispatch");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN);
}

#[test]
fn shared_images_have_isolated_single_use_vm_instances() {
    let image = Arc::new(verified(
        vec![return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        0,
    ));
    let mut first = RegisterVm::from_shared(Arc::clone(&image));
    let mut second = RegisterVm::from_shared(image);

    assert_eq!(first.run().expect("first run").value, Value::Unit);
    assert_eq!(second.run().expect("isolated run").value, Value::Unit);
    let repeated = first.run().expect_err("VM instance must be single-use");
    assert_eq!(repeated.code, RUNTIME_VM_SHUTDOWN);
}

#[test]
fn control_flow_counts_only_dispatched_instructions() {
    let result = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::BranchIfFalse, 0, 4),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::Jump, 0, 5),
            abx(Opcode::LoadConstant, 1, 0),
            return_unit(),
        ],
        vec![Constant::Boolean(true), Constant::Integer(9)],
        vec!["root", "test.fpas"],
        2,
    ))
    .expect("branch program should execute");
    assert_eq!(result.1[1], Value::Integer(9));
    assert_eq!(result.2, 5);
}
