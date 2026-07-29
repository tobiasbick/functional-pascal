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
fn executable_validation_rejects_missing_halt() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, location());

    assert_eq!(
        validate_executable(&chunk),
        Err(ExecutableError::MissingHalt)
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
