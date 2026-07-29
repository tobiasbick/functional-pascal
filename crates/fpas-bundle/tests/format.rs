#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "bundle format tests use direct assertions for fixture construction and exact errors"
)]

use fpas_bytecode::{Chunk, Op, SourceLocation};
use fpas_program::{Digest, ProgramIdentity, ProgramImage};

fn encoded_program() -> Vec<u8> {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, SourceLocation::new_with_source(1, 1, 0));
    let image = ProgramImage::new(
        ProgramIdentity {
            compiler_version: "test".to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"source"),
            options_hash: Digest::of(b"options"),
            units: Vec::new(),
        },
        vec!["main.fpas".to_string()],
        chunk,
    )
    .expect("test image must be valid");
    fpas_program::encode(&image).expect("test image must encode")
}

#[test]
fn round_trip_preserves_runner_program_and_name() {
    let program = encoded_program();
    let bundled =
        fpas_bundle::encode(b"native-runner", &program, "hello").expect("bundle must encode");
    let decoded = fpas_bundle::decode(&bundled).expect("bundle must decode");

    assert!(bundled.starts_with(b"native-runner"));
    assert_eq!(decoded.name, "hello");
    assert_eq!(decoded.image, program);
}

#[test]
fn decoder_rejects_truncated_footer_and_invalid_program() {
    assert_eq!(
        fpas_bundle::decode(b"runner").unwrap_err(),
        fpas_bundle::BundleError::MissingFooter
    );
    assert!(matches!(
        fpas_bundle::encode(b"runner", b"invalid", "hello"),
        Err(fpas_bundle::BundleError::Program(_))
    ));
}

#[test]
fn encoder_rejects_empty_and_oversized_names() {
    let program = encoded_program();

    assert_eq!(
        fpas_bundle::encode(b"runner", &program, " ").unwrap_err(),
        fpas_bundle::BundleError::EmptyName
    );
    assert_eq!(
        fpas_bundle::encode(b"runner", &program, &"x".repeat(4097)).unwrap_err(),
        fpas_bundle::BundleError::NameTooLong(4097)
    );
}

#[test]
fn decoder_rejects_footer_lengths_outside_the_executable() {
    let program = encoded_program();
    let mut bundled =
        fpas_bundle::encode(b"runner", &program, "hello").expect("bundle must encode");
    let image_length_offset = bundled.len() - 12;
    bundled[image_length_offset..image_length_offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());

    assert_eq!(
        fpas_bundle::decode(&bundled).unwrap_err(),
        fpas_bundle::BundleError::InvalidLengths
    );
}
