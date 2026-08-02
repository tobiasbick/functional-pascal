#![expect(
    clippy::expect_used,
    reason = "bytecode fixtures use expect for compact constant setup"
)]

use fpas_bytecode::{Chunk, ExecutableError, Op, SourceLocation, Value, validate_executable};

fn location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[test]
fn executable_validation_accepts_complete_chunk() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Integer(42)).expect("constant");
    chunk.emit(Op::Constant(constant), location());
    chunk.emit(Op::Halt, location());

    assert_eq!(validate_executable(&chunk), Ok(()));
}

#[test]
fn executable_validation_accepts_function_code_after_initial_halt() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, location());
    let function = chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());
    chunk.insert_function("demo.run", function, 0);

    assert_eq!(validate_executable(&chunk), Ok(()));
}

#[test]
fn executable_validation_accepts_conditional_loop_with_halt_exit() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::JumpIfTrue(0), location());
    chunk.emit(Op::Halt, location());

    assert_eq!(validate_executable(&chunk), Ok(()));
}

#[test]
fn executable_validation_accepts_jump_over_function_region() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(3), location());
    let function = chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());
    chunk.emit(Op::Halt, location());
    chunk.insert_function("demo.run", function, 0);

    assert_eq!(validate_executable(&chunk), Ok(()));
}

#[test]
fn executable_validation_rejects_empty_chunk_without_entry_exit() {
    assert_eq!(
        validate_executable(&Chunk::new()),
        Err(ExecutableError::MissingEntryExit)
    );
}

#[test]
fn executable_validation_rejects_entry_fallthrough() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, location());

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::EntryFallthrough { instruction: 0 })
    );
}

#[test]
fn executable_validation_rejects_unreachable_halt_after_entry_loop() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(0), location());
    chunk.emit(Op::Halt, location());

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::MissingEntryExit)
    );
}

#[test]
fn executable_validation_accepts_return_from_entry() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());

    assert_eq!(validate_executable(&chunk), Ok(()));
}

#[test]
fn executable_validation_rejects_entry_jump_into_function_region() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(1), location());
    let function = chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());
    chunk.emit(Op::Halt, location());
    chunk.insert_function("demo.run", function, 0);

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::EntryFunctionRegion {
            instruction: 0,
            target: 1,
        })
    );
}

#[test]
fn executable_validation_rejects_invalid_constant_index() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Constant(0), location());
    chunk.emit(Op::Halt, location());

    assert!(matches!(
        validate_executable(&chunk),
        Err(ExecutableError::ConstantIndex {
            instruction: 0,
            index: 0,
            constants: 0,
        })
    ));
}

#[test]
fn executable_validation_rejects_target_after_code() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(2), location());
    chunk.emit(Op::Halt, location());

    assert!(matches!(
        validate_executable(&chunk),
        Err(ExecutableError::CodeTarget {
            instruction: 0,
            target: 2,
            code: 2,
        })
    ));
}

#[test]
fn executable_validation_rejects_unknown_intrinsic() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::MAX), location());
    chunk.emit(Op::Halt, location());

    assert!(matches!(
        validate_executable(&chunk),
        Err(ExecutableError::Intrinsic {
            instruction: 0,
            intrinsic: u16::MAX,
        })
    ));
}

#[test]
fn executable_validation_rejects_non_string_name_constants() {
    let cases = [
        (Op::GetGlobal(0), "global name"),
        (Op::SetGlobal(0), "global name"),
        (Op::GlobalIndexSet(0, 1), "global name"),
        (Op::Call(0, 0), "function name"),
        (Op::MakeClosure(0, 0), "function name"),
        (Op::MakeRecord(0, 0), "record type name"),
        (Op::FieldGet(0), "record field name"),
        (Op::FieldSet(0), "record field name"),
        (Op::MakeEnum(0, 0, 0), "enum type name"),
        (Op::IsVariant(0, 0), "enum type name"),
    ];

    for (op, operand) in cases {
        let mut chunk = Chunk::new();
        let index = chunk
            .add_constant(Value::Integer(7))
            .expect("integer constant");
        assert_eq!(index, 0);
        chunk.emit(op, location());
        chunk.emit(Op::Halt, location());

        assert_eq!(
            validate_executable(&chunk),
            Err(ExecutableError::NameConstantType {
                instruction: 0,
                index: 0,
                operand,
                actual: "integer",
            }),
            "opcode {op:?}"
        );
    }
}

#[test]
fn executable_validation_checks_enum_variant_name_constant() {
    for op in [Op::MakeEnum(0, 1, 0), Op::IsVariant(0, 1)] {
        let mut chunk = Chunk::new();
        chunk
            .add_constant(Value::Str("demo.state".into()))
            .expect("type name");
        chunk
            .add_constant(Value::Boolean(false))
            .expect("non-string variant");
        chunk.emit(op, location());
        chunk.emit(Op::Halt, location());

        assert_eq!(
            validate_executable(&chunk),
            Err(ExecutableError::NameConstantType {
                instruction: 0,
                index: 1,
                operand: "enum variant name",
                actual: "boolean",
            })
        );
    }
}

#[test]
fn executable_validation_rejects_unknown_direct_call() {
    let mut chunk = Chunk::new();
    let name = chunk
        .add_constant(Value::Str("demo.missing".into()))
        .expect("function name");
    chunk.emit(Op::Call(name, 0), location());
    chunk.emit(Op::Halt, location());

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::UnknownFunction {
            instruction: 0,
            name: "demo.missing".to_string(),
        })
    );
}

#[test]
fn executable_validation_rejects_unknown_closure_target() {
    let mut chunk = Chunk::new();
    let name = chunk
        .add_constant(Value::Str("demo.missing".into()))
        .expect("function name");
    chunk.emit(Op::MakeClosure(name, 0), location());
    chunk.emit(Op::Halt, location());

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::UnknownFunction {
            instruction: 0,
            name: "demo.missing".to_string(),
        })
    );
}

#[test]
fn executable_validation_rejects_direct_call_arity_mismatch() {
    let mut chunk = Chunk::new();
    let name = chunk
        .add_constant(Value::Str("Demo.Run".into()))
        .expect("function name");
    chunk.emit(Op::Jump(3), location());
    let function = chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());
    chunk.emit(Op::Call(name, 1), location());
    chunk.emit(Op::Halt, location());
    chunk.insert_function("demo.run", function, 2);

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::FunctionArity {
            instruction: 3,
            name: "Demo.Run".to_string(),
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn executable_validation_accepts_known_call_and_closure_targets() {
    let mut chunk = Chunk::new();
    let name = chunk
        .add_constant(Value::Str("Demo.Run".into()))
        .expect("function name");
    chunk.emit(Op::Jump(3), location());
    let function = chunk.emit(Op::Unit, location());
    chunk.emit(Op::Return, location());
    chunk.emit(Op::Call(name, 2), location());
    chunk.emit(Op::MakeClosure(name, 0), location());
    chunk.emit(Op::Halt, location());
    chunk.insert_function("demo.run", function, 2);

    assert_eq!(validate_executable(&chunk), Ok(()));
}
