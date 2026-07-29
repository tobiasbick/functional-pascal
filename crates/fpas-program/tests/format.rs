#![expect(
    clippy::expect_used,
    reason = "binary format fixtures use expect for compact assertions"
)]

use fpas_bytecode::{Chunk, Op, SourceLocation, Value};
use fpas_program::{
    Digest, FormatError, LinkedUnitIdentity, ProgramIdentity, ProgramImage, decode, encode,
};

fn program_image() -> ProgramImage {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Integer(42)).expect("constant");
    chunk.emit(
        Op::Constant(constant),
        SourceLocation::new_with_source(3, 2, 0),
    );
    chunk.emit(Op::PrintLn, SourceLocation::new_with_source(3, 2, 0));
    chunk.emit(Op::Halt, SourceLocation::new_with_source(4, 1, 0));
    chunk.insert_function("demo.main", 0, 0);

    ProgramImage::new(
        ProgramIdentity {
            compiler_version: "0.0.1-test".to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"program Demo;"),
            options_hash: Digest::of(b"default-options"),
            units: vec![LinkedUnitIdentity {
                unit_name: "std.console".to_string(),
                object_hash: Digest::of(b"console-object"),
            }],
        },
        vec!["src/main.fpas".to_string()],
        chunk,
    )
    .expect("program image")
}

#[test]
fn format_round_trip_preserves_complete_program_image() {
    let image = program_image();
    let bytes = encode(&image).expect("encoding");
    let decoded = decode(&bytes).expect("decoding");

    assert_eq!(decoded.identity(), image.identity());
    assert_eq!(decoded.source_paths(), image.source_paths());
    assert_eq!(decoded.chunk().code(), image.chunk().code());
    assert_eq!(decoded.chunk().constants(), image.chunk().constants());
    assert_eq!(decoded.chunk().locations(), image.chunk().locations());
    assert_eq!(decoded.chunk().functions(), image.chunk().functions());
}

#[test]
fn format_encoding_is_deterministic() {
    let image = program_image();

    assert_eq!(
        encode(&image).expect("first encoding"),
        encode(&image).expect("second encoding")
    );
}

#[test]
fn decoder_rejects_invalid_magic() {
    let mut bytes = encode(&program_image()).expect("encoding");
    bytes[0] ^= 0xff;

    assert_eq!(decode(&bytes).err(), Some(FormatError::InvalidMagic));
}

#[test]
fn decoder_rejects_unsupported_format_version() {
    let mut bytes = encode(&program_image()).expect("encoding");
    bytes[8..10].copy_from_slice(&99_u16.to_le_bytes());

    assert_eq!(
        decode(&bytes).err(),
        Some(FormatError::UnsupportedVersion(99))
    );
}

#[test]
fn decoder_rejects_every_truncated_prefix_without_panicking() {
    let bytes = encode(&program_image()).expect("encoding");

    for length in 0..bytes.len() {
        assert!(
            decode(&bytes[..length]).is_err(),
            "prefix of {length} bytes must fail"
        );
    }
}

#[test]
fn decoder_rejects_trailing_bytes() {
    let mut bytes = encode(&program_image()).expect("encoding");
    bytes.extend_from_slice(&[1, 2]);

    assert_eq!(decode(&bytes).err(), Some(FormatError::TrailingBytes(2)));
}

#[test]
fn decoder_rejects_payload_hash_mismatch() {
    let mut bytes = encode(&program_image()).expect("encoding");
    let last = bytes.last_mut().expect("payload byte");
    *last ^= 0xff;

    assert_eq!(decode(&bytes).err(), Some(FormatError::PayloadHashMismatch));
}

#[test]
fn encoder_rejects_oversized_compiler_identity() {
    let base = program_image();
    let mut identity = base.identity().clone();
    identity.compiler_version = "x".repeat(1024 * 1024 + 1);
    let image = ProgramImage::new(identity, base.source_paths().to_vec(), base.chunk().clone())
        .expect("oversized identity remains structurally valid");

    assert!(matches!(
        encode(&image),
        Err(FormatError::LimitExceeded {
            field: "compiler_version",
            ..
        })
    ));
}
