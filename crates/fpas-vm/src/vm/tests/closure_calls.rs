//! Copying closure captures from the caller's function register into the callee frame.

use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};

use crate::vm::Vm;

use super::calls::{FunctionSpec, abc, image};
use super::support::abx;

#[test]
fn closure_passed_as_its_own_overlapping_argument_keeps_its_captures() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeClosure, 1, 2, 0, 1),
            // The closure is also the last argument, so the callee frame starts on it.
            abc(Opcode::CallValue, 0, 1, 1, 1),
            abc(Opcode::Return, 0, 0, 0, 0),
            abc(Opcode::AddIntegerImm, 1, 1, 2, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        vec![Constant::Integer(40)],
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 5,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 5,
                end: 7,
                arity: 1,
                captures: 1,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );

    let result = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect("closure call should succeed");
    assert_eq!(
        result.value.try_to_data().expect("pure return value"),
        Value::Integer(42)
    );
}

#[test]
fn dynamic_calls_still_reject_incorrect_arity_and_missing_captures() {
    for (arity, captures, message) in [
        (1, 0, "Call arity mismatch"),
        (0, 1, "Closure capture mismatch"),
    ] {
        let executable = image(
            vec![
                abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
                abc(Opcode::CallValue, fpas_bytecode::NO_REGISTER, 0, 0, 0),
                abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
                abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            ],
            Vec::new(),
            &[
                FunctionSpec {
                    start: 0,
                    end: 1,
                    arity: 0,
                    captures: 0,
                    registers: 0,
                    returns: ReturnConvention::Unit,
                },
                FunctionSpec {
                    start: 1,
                    end: 3,
                    arity: 1,
                    captures: 0,
                    registers: 1,
                    returns: ReturnConvention::Unit,
                },
                FunctionSpec {
                    start: 3,
                    end: 4,
                    arity,
                    captures,
                    registers: 1,
                    returns: ReturnConvention::Unit,
                },
            ],
        );
        let function = Value::function(
            executable
                .function_target(FunctionId::new(2))
                .expect("target"),
            Vec::new(),
        );
        let error = Vm::new(executable)
            .call(FunctionId::new(1), vec![function.into()])
            .expect_err("dynamic signature must still be checked");
        assert!(error.message.contains(message), "{error:?}");
    }
}

#[test]
fn capture_windows_preserve_initialization_aliases_and_release_all_references() {
    use std::sync::Arc;

    use crate::vm::worker::Worker;

    for argument_base in [0, 2] {
        let executable = image(
            vec![
                abc(
                    Opcode::CallValue,
                    fpas_bytecode::NO_REGISTER,
                    2,
                    argument_base,
                    1,
                ),
                abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
                abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            ],
            Vec::new(),
            &[
                FunctionSpec {
                    start: 0,
                    end: 2,
                    arity: 0,
                    captures: 0,
                    registers: 3,
                    returns: ReturnConvention::Unit,
                },
                FunctionSpec {
                    start: 2,
                    end: 3,
                    arity: 1,
                    captures: 4,
                    registers: 6,
                    returns: ReturnConvention::Unit,
                },
            ],
        );
        let mut worker = Worker::new(Arc::new(executable)).expect("worker");
        let cell = worker.domain.new_cell(Value::Integer(7)).expect("cell");
        let weak = cell.downgrade();
        let function = Value::task_owned_function(
            worker.function_target(FunctionId::new(1)).expect("target"),
            vec![
                Value::Integer(11),
                Value::Real(2.5),
                Value::Boolean(true),
                Value::Cell(cell),
            ],
            worker.domain.id().clone(),
        );
        worker.store_register(2, function).expect("caller closure");
        worker.dispatch_one().expect("enter closure");
        assert_eq!(worker.function, FunctionId::new(1));
        let start = worker.base + 1;
        assert!((start..start + 4).all(|index| worker.register_is_initialized(index)));
        assert!(
            !worker.register_is_initialized(start + 4),
            "unused callee slot"
        );
        assert_eq!(
            worker.registers[start..start + 3],
            [Value::Integer(11), Value::Real(2.5), Value::Boolean(true)]
        );
        let Value::Function(function) = &worker.registers[2] else {
            panic!("caller closure remains available while entering the callee");
        };
        let (Value::Cell(original), Value::Cell(captured)) =
            (&function.captures[3], &worker.registers[start + 3])
        else {
            panic!("mutable capture retains its cell");
        };
        assert!(original.ptr_eq(captured));
        captured
            .replace(&worker.domain, Value::Integer(42))
            .expect("write capture");
        assert_eq!(
            original.read(&worker.domain).expect("read alias"),
            Value::Integer(42)
        );
        worker.dispatch_one().expect("return from closure");
        worker.dispatch_one().expect("return from root");
        worker.retire_execution();
        assert_eq!(
            worker.domain.registered_count(),
            0,
            "all cell owners released"
        );
        assert!(weak.upgrade().is_none());
    }
}
