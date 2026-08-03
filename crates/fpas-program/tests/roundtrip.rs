#![expect(
    clippy::expect_used,
    reason = "program image fixtures use expect for compact assertions"
)]

use std::sync::{Arc, Mutex};

use fpas_bytecode::{Chunk, ExecutableError, Op, SourceLocation, Value};
use fpas_program::{Digest, ImageError, ProgramIdentity, ProgramImage, decode, encode};

fn identity() -> ProgramIdentity {
    ProgramIdentity {
        compiler_version: "0.0.1-test".to_string(),
        bytecode_version: fpas_bytecode::BYTECODE_VERSION,
        source_hash: Digest::of(b"program Demo;"),
        options_hash: Digest::of(b"default-options"),
        units: Vec::new(),
    }
}

fn printable_chunk() -> Chunk {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Integer(42)).expect("constant");
    chunk.emit(Op::Constant(constant), SourceLocation::new(1, 1));
    chunk.emit(Op::PrintLn, SourceLocation::new(1, 1));
    chunk.emit(Op::Halt, SourceLocation::new(2, 1));
    chunk
}

#[test]
fn decoded_program_executes_with_the_same_vm_result() {
    let image = ProgramImage::new(identity(), vec!["main.fpas".to_string()], printable_chunk())
        .expect("image");
    let decoded = decode(&encode(&image).expect("encoding")).expect("decoding");
    let mut vm = fpas_vm::Vm::new(decoded.into_chunk());

    vm.run().expect("execution");

    assert_eq!(vm.output().lines, vec!["42"]);
}

#[test]
fn image_rejects_runtime_only_constant() {
    let mut chunk = Chunk::new();
    chunk
        .add_constant(Value::Cell(Arc::new(Mutex::new(Value::Integer(1)))))
        .expect("constant");
    chunk.emit(Op::Halt, SourceLocation::new(1, 1));

    assert!(matches!(
        ProgramImage::new(identity(), vec!["main.fpas".to_string()], chunk),
        Err(ImageError::PersistentValue(_))
    ));
}

#[test]
fn image_rejects_invalid_constant_operand() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Constant(0), SourceLocation::new(1, 1));
    chunk.emit(Op::Halt, SourceLocation::new(1, 1));

    assert!(matches!(
        ProgramImage::new(identity(), vec!["main.fpas".to_string()], chunk),
        Err(ImageError::Executable(ExecutableError::ConstantIndex {
            instruction: 0,
            ..
        }))
    ));
}

#[test]
fn image_rejects_source_id_outside_path_table() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, SourceLocation::new_with_source(1, 1, 1));

    assert_eq!(
        ProgramImage::new(identity(), vec!["main.fpas".to_string()], chunk).err(),
        Some(ImageError::SourceId {
            instruction: 0,
            source_id: 1,
            source_paths: 1,
        })
    );
}

#[test]
fn image_rejects_absolute_source_path() {
    assert!(matches!(
        ProgramImage::new(
            identity(),
            vec![
                std::env::current_dir()
                    .expect("current directory")
                    .join("main.fpas")
                    .to_string_lossy()
                    .into_owned()
            ],
            printable_chunk(),
        ),
        Err(ImageError::AbsoluteSourcePath(_))
    ));
}

#[test]
fn image_rejects_absolute_source_paths_from_every_host_syntax() {
    for path in [
        "/workspace/main.fpas",
        r"C:\workspace\main.fpas",
        "C:/workspace/main.fpas",
        r"\\server\share\main.fpas",
        r"\workspace\main.fpas",
    ] {
        assert!(matches!(
            ProgramImage::new(identity(), vec![path.to_string()], printable_chunk()),
            Err(ImageError::AbsoluteSourcePath(rejected)) if rejected == path
        ));
    }
}

#[test]
fn image_accepts_relative_source_paths_from_common_syntaxes() {
    for path in ["src/main.fpas", r"src\main.fpas", "C:main.fpas"] {
        assert!(
            ProgramImage::new(identity(), vec![path.to_string()], printable_chunk()).is_ok(),
            "relative path `{path}` must remain valid"
        );
    }
}

#[test]
fn image_rejects_incompatible_bytecode_version() {
    let mut incompatible = identity();
    incompatible.bytecode_version += 1;

    assert_eq!(
        ProgramImage::new(
            incompatible,
            vec!["main.fpas".to_string()],
            printable_chunk(),
        )
        .err(),
        Some(ImageError::BytecodeVersion {
            image: fpas_bytecode::BYTECODE_VERSION + 1,
            runtime: fpas_bytecode::BYTECODE_VERSION,
        })
    );
}
