mod vm;

pub use vm::{Vm, VmError, VmOutput};

#[cfg(test)]
mod tests {
    use super::Vm;
    use fpas_bytecode::{Chunk, Intrinsic, Op, SourceLocation, Value};
    use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_NUMERIC_DOMAIN_ERROR};

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn emit_constant(chunk: &mut Chunk, value: Value) {
        let idx = chunk.add_constant(value);
        chunk.emit(Op::Constant(idx), loc());
    }

    fn build_zero_arg_function_chunk(
        function_name: &str,
        main: impl FnOnce(&mut Chunk),
        body: impl FnOnce(&mut Chunk),
    ) -> Chunk {
        let mut chunk = Chunk::new();
        main(&mut chunk);
        chunk.emit(Op::Halt, loc());

        let code_start = chunk.len();
        chunk.functions
            .insert(function_name.to_string(), (code_start, 0));
        body(&mut chunk);
        chunk
    }

    fn run_err(chunk: Chunk) -> fpas_diagnostics::Diagnostic {
        let mut vm = Vm::new(chunk);
        vm.run().expect_err("VM should return an error")
    }

    #[test]
    fn malformed_call_reports_error_instead_of_panicking() {
        let mut chunk = Chunk::new();
        let name_idx = chunk.add_constant(Value::Str("NeedArg".to_string()));
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
    fn try_receive_on_closed_empty_channel_returns_none() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Intrinsic(Intrinsic::ChannelMake as u16), loc());
        chunk.emit(Op::Dup, loc());
        chunk.emit(Op::Intrinsic(Intrinsic::ChannelClose as u16), loc());
        chunk.emit(Op::Intrinsic(Intrinsic::ChannelTryRecv as u16), loc());
        chunk.emit(Op::PrintLn, loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        vm.run()
            .expect("TryReceive on a closed, empty channel should return None");
        assert_eq!(vm.output().lines, vec!["None"]);
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
        let type_idx = chunk.add_constant(Value::Str("Point".to_string()));
        emit_constant(&mut chunk, Value::Str("x".to_string()));
        emit_constant(&mut chunk, Value::Integer(1));
        chunk.emit(Op::MakeRecord(type_idx, 1), loc());
        emit_constant(&mut chunk, Value::Integer(2));

        let missing_field_idx = chunk.add_constant(Value::Str("y".to_string()));
        chunk.emit(Op::FieldSet(missing_field_idx), loc());
        chunk.emit(Op::Halt, loc());

        let mut vm = Vm::new(chunk);
        assert!(
            vm.run().is_err(),
            "FieldSet on an unknown field should return a VM error"
        );
    }
}
