//! Prepared callbacks preserve invocation limits, initialization and value lifetimes.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`.

use std::sync::Arc;

use crate::execution::vm::VmError;
use crate::execution::vm::frame::{MAX_CALL_DEPTH, MAX_REGISTER_SLOTS};
use crate::execution::vm::worker::Worker;
use crate::{
    ArrayIntrinsic, FunctionId, Intrinsic, Opcode, ReturnConvention, SourceLocation, Value,
};

use super::calls::{FunctionSpec, abc, image};

fn worker(arity: u8, captures: u16) -> Worker {
    Worker::new(Arc::new(image(
        vec![
            abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, 0, 0, 0, 0),
        ],
        Vec::new(),
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 2,
                arity,
                captures,
                registers: u16::from(arity) + captures + 2,
                returns: ReturnConvention::Value,
            },
        ],
    )))
    .expect("callback worker")
}

fn start(worker: &mut Worker, function: Value, values: Vec<Value>) -> Result<(), VmError> {
    let access = worker.execution_access()?;
    let result = worker
        .execute_callback_intrinsic(
            Intrinsic::Array(ArrayIntrinsic::Map),
            &[Value::Array(values.into()), function],
            SourceLocation::new(1, 1),
            Some(0),
        )
        .map(|_| ());
    worker.publish_execution(&[])?;
    drop(access);
    result
}

#[test]
fn prepared_callbacks_recheck_current_stack_and_register_limits() {
    for register_limit in [false, true] {
        let mut worker = worker(1, 0);
        let function = Value::function(
            worker.function_target(FunctionId::new(1)).unwrap(),
            Vec::new(),
        );
        start(
            &mut worker,
            function,
            vec![Value::Integer(11), Value::Integer(12)],
        )
        .unwrap();
        let frame = worker.call_stack[0];
        worker.dispatch_one().expect("first callback returns");
        assert!(worker.callback_resume_pending());
        assert!(worker.call_stack.is_empty());
        if register_limit {
            worker.activate_registers(MAX_REGISTER_SLOTS - 1);
        } else {
            worker.call_stack.resize(MAX_CALL_DEPTH, frame);
        }
        let before = (
            worker.function,
            worker.ip,
            worker.base,
            worker.call_stack.len(),
            worker.registers.len(),
        );
        let error = worker
            .resume_callback_continuation()
            .expect_err("current limit must apply");
        let message = if register_limit {
            "Register stack overflow"
        } else {
            "Call stack overflow"
        };
        assert_eq!(error.message, message);
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR
        );
        assert_eq!(
            before,
            (
                worker.function,
                worker.ip,
                worker.base,
                worker.call_stack.len(),
                worker.registers.len()
            ),
            "failed entry must not activate a frame"
        );
    }
}

#[test]
fn callback_capture_validation_still_waits_for_an_actual_invocation() {
    for empty in [true, false] {
        let mut worker = worker(1, 1);
        let function = Value::function(
            worker.function_target(FunctionId::new(1)).unwrap(),
            Vec::new(),
        );
        worker.store_register(0, Value::Integer(99)).unwrap();
        let result = start(
            &mut worker,
            function,
            if empty {
                Vec::new()
            } else {
                vec![Value::Integer(1)]
            },
        );
        if empty {
            result.expect("empty map never enters its malformed capture frame");
            assert_eq!(worker.registers[0], Value::Array(Vec::new().into()));
        } else {
            let error = result.expect_err("nonempty map checks capture shape");
            assert_eq!(error.message, "Closure capture mismatch: expected 1, got 0");
            assert_eq!(worker.registers[0], Value::Integer(99));
        }
        assert!(worker.call_stack.is_empty());
        assert_eq!(worker.base, 0);
    }
}

#[test]
fn repeated_callback_frames_preserve_capture_aliases_and_clear_locals() {
    let mut worker = worker(1, 2);
    // Direct fixture mutations need the same execution lease as dispatched instructions.
    let access = worker.execution_access().unwrap();
    let captured = worker.domain.new_cell(Value::Integer(41)).unwrap();
    let captured_weak = captured.downgrade();
    let function = Value::task_owned_function(
        worker.function_target(FunctionId::new(1)).unwrap(),
        vec![Value::Integer(7), Value::Cell(captured)],
        worker.domain.id().clone(),
    );
    start(
        &mut worker,
        function,
        vec![Value::Integer(11), Value::Integer(12)],
    )
    .unwrap();
    let base = worker.base;
    assert_eq!(worker.registers[base], Value::Integer(11));
    assert_eq!(worker.registers[base + 1], Value::Integer(7));
    let Value::Cell(captured) = &worker.registers[base + 2] else {
        panic!("capture cell")
    };
    captured
        .replace(&worker.domain, Value::Integer(42))
        .unwrap();
    let local = worker.domain.new_cell(Value::Unit).unwrap();
    let local_weak = local.downgrade();
    worker.store_register(base + 3, Value::Cell(local)).unwrap();
    worker.publish_execution(&[]).unwrap();
    drop(access);
    worker.dispatch_one().expect("return releases first locals");
    assert!(local_weak.upgrade().is_none());
    worker
        .resume_callback_continuation()
        .expect("reuse prepared entry");
    assert_eq!(worker.base, base);
    assert_eq!(worker.registers[base], Value::Integer(12));
    let Value::Cell(captured) = &worker.registers[base + 2] else {
        panic!("retained cell")
    };
    assert_eq!(captured.read(&worker.domain).unwrap(), Value::Integer(42));
    assert!((base..base + 3).all(|slot| worker.register_is_initialized(slot)));
    for slot in base + 3..base + 5 {
        assert!(!worker.register_is_initialized(slot));
        assert_eq!(worker.registers[slot], Value::Unit);
    }
    worker.dispatch_one().expect("second return");
    worker
        .dispatch_one()
        .expect("complete callback operation and publish roots");
    assert!(worker.callback_continuations.is_empty());
    assert_eq!(
        worker.registers[0],
        Value::Array(vec![Value::Integer(11), Value::Integer(12)].into())
    );
    assert!(captured_weak.upgrade().is_none());
    assert_eq!(worker.domain.registered_count(), 0);
}

#[test]
fn bound_callback_receivers_stay_before_arguments_and_survive_in_results() {
    let mut worker = worker(2, 0);
    let access = worker.execution_access().unwrap();
    let receiver = worker.domain.new_cell(Value::Integer(42)).unwrap();
    let weak = receiver.downgrade();
    let function = Value::bound_function(
        worker.function_target(FunctionId::new(1)).unwrap(),
        Value::Cell(receiver),
    );
    start(
        &mut worker,
        function,
        vec![Value::Integer(11), Value::Integer(12)],
    )
    .unwrap();
    drop(access);
    for argument in [11, 12] {
        let base = worker.base;
        let Value::Cell(receiver) = &worker.registers[base] else {
            panic!("bound receiver")
        };
        assert_eq!(receiver.read(&worker.domain).unwrap(), Value::Integer(42));
        assert_eq!(worker.registers[base + 1], Value::Integer(argument));
        assert!(worker.register_is_initialized(base));
        assert!(worker.register_is_initialized(base + 1));
        assert!(!worker.register_is_initialized(base + 2));
        worker
            .dispatch_one()
            .expect("bound callback returns receiver");
        worker.dispatch_one().expect("advance callback operation");
    }
    assert!(worker.callback_continuations.is_empty());
    let Value::Array(values) = &worker.registers[0] else {
        panic!("mapped receivers")
    };
    let (Value::Cell(first), Value::Cell(second)) = (&values[0], &values[1]) else {
        panic!("receiver aliases")
    };
    assert!(first.ptr_eq(second));
    assert_eq!(worker.domain.registered_count(), 1);
    worker.store_register(0, Value::Unit).unwrap();
    worker.publish_execution(&[]).unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(worker.domain.registered_count(), 0);
}
