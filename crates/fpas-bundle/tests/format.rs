#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "bundle format tests use direct assertions for fixture construction and exact errors"
)]

use fpas_program::{Digest, ProgramIdentity, ProgramImage};

fn encoded_program() -> Vec<u8> {
    let (program, diagnostics) = fpas_parser::parse("program BundleFixture; begin end.");
    assert!(diagnostics.is_empty());
    let executable = fpas_compiler::compile(&program).expect("fixture must compile");
    let image = ProgramImage::new(
        ProgramIdentity {
            compiler_version: "test".to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"source"),
            options_hash: Digest::of(b"options"),
            units: Vec::new(),
        },
        vec!["main.fpas".to_string()],
        vec![Digest::of(b"source")],
        executable,
    )
    .expect("test image must be valid");
    fpas_program::encode(&image).expect("test image must encode")
}

fn decode_error(executable: &[u8]) -> fpas_bundle::BundleError {
    match fpas_bundle::decode(executable) {
        Ok(_) => panic!("bundle must be rejected"),
        Err(error) => error,
    }
}

fn footer_start(executable: &[u8]) -> usize {
    executable.len() - 24
}

fn encoded_bundle(name: &str) -> Vec<u8> {
    fpas_bundle::encode(b"native-runner", &encoded_program(), name).expect("bundle must encode")
}

#[test]
fn round_trip_preserves_runner_program_and_name() {
    let program = encoded_program();
    let bundled =
        fpas_bundle::encode(b"native-runner", &program, "hello").expect("bundle must encode");
    let decoded = fpas_bundle::decode(&bundled).expect("bundle must decode");

    assert!(bundled.starts_with(b"native-runner"));
    assert_eq!(decoded.name, "hello");
    assert_eq!(
        fpas_program::encode(&decoded.image).expect("decoded image must encode"),
        program
    );
}

#[test]
fn decoder_rejects_truncated_footer_and_invalid_program() {
    assert_eq!(
        decode_error(b"runner"),
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
fn name_length_boundary_is_measured_in_utf8_bytes() {
    let program = encoded_program();
    let maximum_name = "x".repeat(4096);

    let bundled =
        fpas_bundle::encode(b"runner", &program, &maximum_name).expect("limit must be accepted");
    assert_eq!(
        fpas_bundle::decode(&bundled)
            .expect("limit bundle must decode")
            .name,
        maximum_name
    );
    assert_eq!(
        fpas_bundle::encode(b"runner", &program, &format!("{}é", "x".repeat(4095))).unwrap_err(),
        fpas_bundle::BundleError::NameTooLong(4097)
    );
}

#[test]
fn decoder_rejects_unsupported_version_and_reserved_data() {
    let mut unsupported = encoded_bundle("hello");
    let footer = footer_start(&unsupported);
    unsupported[footer + 8..footer + 10].copy_from_slice(&2_u16.to_le_bytes());
    assert_eq!(
        decode_error(&unsupported),
        fpas_bundle::BundleError::UnsupportedVersion(2)
    );

    let mut reserved = encoded_bundle("hello");
    let footer = footer_start(&reserved);
    reserved[footer + 10] = 1;
    assert_eq!(
        decode_error(&reserved),
        fpas_bundle::BundleError::ReservedData
    );
}

#[test]
fn decoder_rejects_invalid_utf8_name() {
    let mut bundled = encoded_bundle("x");
    let name_offset = footer_start(&bundled) - 1;
    bundled[name_offset] = 0xff;

    assert_eq!(
        decode_error(&bundled),
        fpas_bundle::BundleError::InvalidName
    );
}

#[test]
fn decoder_preserves_corrupt_program_diagnostic() {
    let mut bundled = encoded_bundle("hello");
    bundled[b"native-runner".len()] ^= 0xff;

    let fpas_bundle::BundleError::Program(message) = decode_error(&bundled) else {
        panic!("corrupt embedded image must report a program error");
    };
    assert!(!message.is_empty());
}

#[test]
fn decoder_rejects_footer_lengths_outside_the_executable() {
    let program = encoded_program();
    let mut bundled =
        fpas_bundle::encode(b"runner", &program, "hello").expect("bundle must encode");
    let image_length_offset = bundled.len() - 12;
    bundled[image_length_offset..image_length_offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());

    assert_eq!(
        decode_error(&bundled),
        fpas_bundle::BundleError::InvalidLengths
    );
}

#[test]
fn bundle_format_matches_golden_bytes() {
    let bundled = encoded_bundle("golden");

    assert_eq!(
        format!("{:?}", fpas_program::Digest::of(bundled)),
        "c29786cb616dc6c4487daa97ed94f1ea0849fb0d14d957a2dae677431d2e4ff7"
    );
}
