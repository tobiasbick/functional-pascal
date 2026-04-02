#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "VM tests use expect to keep low-level bytecode assertions focused on behavior"
    )
)]

mod vm;

pub use vm::{Vm, VmError, VmOutput};

#[cfg(test)]
mod tests {
    use super::Vm;
    use crate::vm::{CallFrame, SharedState, Worker};
    use fpas_bytecode::{Chunk, Intrinsic, Op, SourceLocation, Value};
    use fpas_diagnostics::codes::{
        INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
        RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_DIVISION_BY_ZERO, RUNTIME_INVALID_TASK,
        RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_UNDEFINED_GLOBAL, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        RUNTIME_WRONG_CALL_ARITY,
    };
    use fpas_std::{Console, KeyInput, TextInput};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Condvar, Mutex, RwLock};

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn emit_constant(chunk: &mut Chunk, value: Value) {
        let idx = chunk
            .add_constant(value)
            .expect("constant should fit in test chunk");
        chunk.emit(Op::Constant(idx), loc());
    }

    fn build_function_chunk(
        function_name: &str,
        arity: u8,
        main: impl FnOnce(&mut Chunk),
        body: impl FnOnce(&mut Chunk),
    ) -> Chunk {
        let mut chunk = Chunk::new();
        main(&mut chunk);
        chunk.emit(Op::Halt, loc());

        let code_start = chunk.len();
        chunk
            .functions
            .insert(function_name.to_string(), (code_start, arity));
        body(&mut chunk);
        chunk
    }

    fn build_zero_arg_function_chunk(
        function_name: &str,
        main: impl FnOnce(&mut Chunk),
        body: impl FnOnce(&mut Chunk),
    ) -> Chunk {
        build_function_chunk(function_name, 0, main, body)
    }

    fn run_err(chunk: Chunk) -> fpas_diagnostics::Diagnostic {
        let mut vm = Vm::new(chunk);
        vm.run().expect_err("VM should return an error")
    }

    fn run_ok_output(chunk: Chunk) -> Vec<String> {
        let mut vm = Vm::new(chunk);
        vm.run().expect("VM should succeed");
        vm.output().lines
    }

    #[test]
    fn malformed_call_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        let name_idx = chunk
            .add_constant(Value::Str("NeedArg".to_string()))
            .expect("constant should fit in test chunk");
        chunk.functions.insert("NeedArg".to_string(), (1, 1));
        chunk.emit(Op::Call(name_idx, 1), loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        assert!(vm.run().is_err(), "malformed call should return a VM error");
    }

    #[test]
    fn malformed_make_array_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::MakeArray(1), loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        assert!(
            vm.run().is_err(),
            "malformed MakeArray should return a VM error"
        );
    }

    #[test]
    fn integer_division_overflow_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(i64::MIN));
        emit_constant(&mut chunk, Value::Integer(-1));
        chunk.emit(Op::DivInt, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }

    #[test]
    fn integer_modulo_overflow_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(i64::MIN));
        emit_constant(&mut chunk, Value::Integer(-1));
        chunk.emit(Op::ModInt, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }

    #[test]
    fn integer_negation_overflow_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(i64::MIN));
        chunk.emit(Op::NegateInt, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }

    #[test]
    fn dynamic_negation_overflow_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(i64::MIN));
        chunk.emit(Op::NegateDyn, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }

    #[test]
    fn real_division_by_zero_reports_error_instead_of_returning_infinity() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Real(1.0));
        emit_constant(&mut chunk, Value::Real(0.0));
        chunk.emit(Op::DivReal, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_DIVISION_BY_ZERO);
    }

    #[test]
    fn dynamic_real_division_by_zero_reports_error() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Real(1.0));
        emit_constant(&mut chunk, Value::Real(0.0));
        chunk.emit(Op::DivDyn, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_DIVISION_BY_ZERO);
    }

    #[test]
    fn set_global_then_get_global_round_trips_value() {
        let mut chunk = Chunk::new();
        let name_idx = chunk
            .add_constant(Value::Str("Answer".to_string()))
            .expect("constant should fit in test chunk");

        emit_constant(&mut chunk, Value::Integer(42));
        chunk.emit(Op::SetGlobal(name_idx), loc());
        chunk.emit(Op::Pop, loc());
        chunk.emit(Op::GetGlobal(name_idx), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(run_ok_output(chunk), vec!["42"]);
    }

    #[test]
    fn set_global_then_get_global_is_case_insensitive() {
        let mut chunk = Chunk::new();
        let set_name_idx = chunk
            .add_constant(Value::Str("Answer".to_string()))
            .expect("constant should fit in test chunk");
        let get_name_idx = chunk
            .add_constant(Value::Str("answer".to_string()))
            .expect("constant should fit in test chunk");

        emit_constant(&mut chunk, Value::Integer(42));
        chunk.emit(Op::SetGlobal(set_name_idx), loc());
        chunk.emit(Op::Pop, loc());
        chunk.emit(Op::GetGlobal(get_name_idx), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(run_ok_output(chunk), vec!["42"]);
    }

    #[test]
    fn get_global_on_missing_name_reports_runtime_error() {
        let mut chunk = Chunk::new();
        let name_idx = chunk
            .add_constant(Value::Str("Missing".to_string()))
            .expect("constant should fit in test chunk");
        chunk.emit(Op::GetGlobal(name_idx), loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_UNDEFINED_GLOBAL);
    }

    #[test]
    fn call_value_executes_function_value_and_returns_result() {
        let function_name = "ReturnNine";
        let chunk = build_zero_arg_function_chunk(
            function_name,
            |chunk| {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: function_name.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::CallValue(0), loc());
                chunk.emit(Op::PrintLn, loc());
            },
            |chunk| {
                emit_constant(chunk, Value::Integer(9));
                chunk.emit(Op::Return, loc());
            },
        );

        assert_eq!(run_ok_output(chunk), vec!["9"]);
    }

    #[test]
    fn call_value_resolves_function_name_case_insensitively() {
        let mut chunk = Chunk::new();
        emit_constant(
            &mut chunk,
            Value::Function {
                name: "ReturnNine".to_string(),
                captures: vec![],
            },
        );
        chunk.emit(Op::CallValue(0), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        let code_start = chunk.len();
        chunk
            .functions
            .insert("returnnine".to_string(), (code_start, 0));
        emit_constant(&mut chunk, Value::Integer(9));
        chunk.emit(Op::Return, loc());

        assert_eq!(run_ok_output(chunk), vec!["9"]);
    }

    #[test]
    fn call_value_with_non_function_reports_operand_type_mismatch() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::CallValue(0), loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }

    #[test]
    fn string_index_get_returns_character_value() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Str("pascal".to_string()));
        emit_constant(&mut chunk, Value::Integer(2));
        chunk.emit(Op::IndexGet, loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(run_ok_output(chunk), vec!["s"]);
    }

    #[test]
    fn array_index_get_with_negative_index_reports_bounds_error() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(7));
        chunk.emit(Op::MakeArray(1), loc());
        emit_constant(&mut chunk, Value::Integer(-1));
        chunk.emit(Op::IndexGet, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }

    #[test]
    fn dict_index_set_adds_new_key_and_makes_it_readable() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::MakeDict(0), loc());
        emit_constant(&mut chunk, Value::Str("language".to_string()));
        emit_constant(&mut chunk, Value::Integer(2024));
        chunk.emit(Op::IndexSet, loc());
        emit_constant(&mut chunk, Value::Str("language".to_string()));
        chunk.emit(Op::IndexGet, loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(run_ok_output(chunk), vec!["2024"]);
    }

    #[test]
    fn dict_index_get_missing_key_reports_runtime_error() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Str("key".to_string()));
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::MakeDict(1), loc());
        emit_constant(&mut chunk, Value::Str("other".to_string()));
        chunk.emit(Op::IndexGet, loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_DICT_KEY_NOT_FOUND);
    }

    #[test]
    fn update_record_overrides_selected_field_and_keeps_others() {
        let mut chunk = Chunk::new();
        let type_idx = chunk
            .add_constant(Value::Str("Point".to_string()))
            .expect("constant should fit in test chunk");
        let x_idx = chunk
            .add_constant(Value::Str("x".to_string()))
            .expect("constant should fit in test chunk");
        let y_idx = chunk
            .add_constant(Value::Str("y".to_string()))
            .expect("constant should fit in test chunk");

        emit_constant(&mut chunk, Value::Str("x".to_string()));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Str("y".to_string()));
        emit_constant(&mut chunk, Value::Integer(2));
        chunk.emit(Op::MakeRecord(type_idx, 2), loc());
        emit_constant(&mut chunk, Value::Str("x".to_string()));
        emit_constant(&mut chunk, Value::Integer(9));
        chunk.emit(Op::UpdateRecord(1), loc());
        chunk.emit(Op::Dup, loc());
        chunk.emit(Op::FieldGet(x_idx), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::FieldGet(y_idx), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(run_ok_output(chunk), vec!["9", "2"]);
    }

    #[test]
    fn update_record_with_unknown_field_reports_runtime_error() {
        let mut chunk = Chunk::new();
        let type_idx = chunk
            .add_constant(Value::Str("Point".to_string()))
            .expect("constant should fit in test chunk");

        emit_constant(&mut chunk, Value::Str("x".to_string()));
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::MakeRecord(type_idx, 1), loc());
        emit_constant(&mut chunk, Value::Str("y".to_string()));
        emit_constant(&mut chunk, Value::Integer(2));
        chunk.emit(Op::UpdateRecord(1), loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }

    #[test]
    fn wait_all_with_non_task_value_reports_operand_type_mismatch() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::MakeArray(1), loc());
        chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }

    #[test]
    fn spawn_task_with_wrong_arity_reports_runtime_error() {
        let function_name = "NeedOneArg";
        let chunk = build_function_chunk(
            function_name,
            1,
            |chunk| {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: function_name.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
            },
            |chunk| {
                emit_constant(chunk, Value::Integer(0));
                chunk.emit(Op::Return, loc());
            },
        );

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_WRONG_CALL_ARITY);
    }

    #[test]
    fn waiting_twice_on_same_task_reports_runtime_error() {
        let function_name = "ReturnSeven";
        let chunk = build_zero_arg_function_chunk(
            function_name,
            |chunk| {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: function_name.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
                chunk.emit(Op::Dup, loc());
                chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
                chunk.emit(Op::Pop, loc());
                chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            },
            |chunk| {
                emit_constant(chunk, Value::Integer(7));
                chunk.emit(Op::Return, loc());
            },
        );

        let err = run_err(chunk);
        assert_eq!(err.code, RUNTIME_INVALID_TASK);
    }

    #[test]
    fn wait_all_keeps_task_result_available_for_wait() {
        let function_name = "ReturnSeven";
        let chunk = build_zero_arg_function_chunk(
            function_name,
            |chunk| {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: function_name.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
                chunk.emit(Op::Dup, loc());
                chunk.emit(Op::MakeArray(1), loc());
                chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
                chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
                chunk.emit(Op::PrintLn, loc());
            },
            |chunk| {
                emit_constant(chunk, Value::Integer(7));
                chunk.emit(Op::Return, loc());
            },
        );

        let mut vm = Vm::new(chunk);
        vm.run().expect("WaitAll followed by Wait should succeed");
        assert_eq!(vm.output().lines, vec!["7"]);
    }

    #[test]
    fn malformed_get_enclosing_reports_error_instead_of_silently_falling_back() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::GetEnclosing(2, 0), loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        assert!(
            vm.run().is_err(),
            "malformed GetEnclosing should return a VM error"
        );
    }

    #[test]
    fn malformed_field_set_missing_field_reports_error() {
        let mut chunk = Chunk::new();
        let type_idx = chunk
            .add_constant(Value::Str("Point".to_string()))
            .expect("constant should fit in test chunk");
        emit_constant(&mut chunk, Value::Str("x".to_string()));
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::MakeRecord(type_idx, 1), loc());
        emit_constant(&mut chunk, Value::Integer(2));

        let missing_field_idx = chunk
            .add_constant(Value::Str("y".to_string()))
            .expect("constant should fit in test chunk");
        chunk.emit(Op::FieldSet(missing_field_idx), loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        assert!(
            vm.run().is_err(),
            "FieldSet on an unknown field should return a VM error"
        );
    }

    #[test]
    fn jump_past_end_reports_internal_vm_error() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Jump(3), loc());
        chunk.emit(Op::Halt, loc());

        let err = run_err(chunk);
        assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    }

    #[test]
    fn pool_tasks_stop_without_side_effects_after_shutdown() {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Str("late".to_string()));
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(SharedState {
            chunk,
            globals: RwLock::new(HashMap::new()),
            task_queue: Mutex::new(Vec::new()),
            task_available: Condvar::new(),
            task_results: Mutex::new(HashMap::new()),
            next_task_id: AtomicU64::new(1),
            console: Mutex::new(Console::new()),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            shutdown: AtomicBool::new(true),
        });

        let mut worker = Worker::new_pool(Arc::clone(&shared));
        worker.load_task(crate::vm::TaskState {
            id: 1,
            ip: 0,
            stack: Vec::new(),
            call_stack: Vec::<CallFrame>::new(),
            retain_result: false,
        });

        worker
            .run()
            .expect("shutdown should stop pool tasks cleanly");

        let output = shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .clone();
        assert!(
            output.lines.is_empty(),
            "pool task should not emit output after shutdown"
        );
        assert!(shared.shutdown.load(Ordering::Acquire));
    }
}
