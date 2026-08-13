#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "program wire-format tests use exact fixture assertions"
)]

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};

use fpas_program::{FormatError, PROGRAM_FORMAT_VERSION, decode, encode};

use common::{payload_start, program_image, refresh_payload_digest};

fn decode_error(bytes: &[u8]) -> FormatError {
    decode(bytes).unwrap_err()
}

#[test]
fn deterministic_round_trip_preserves_every_register_table() {
    let image = program_image();
    let first = encode(&image).expect("first encoding");
    let second = encode(&image).expect("second encoding");
    let decoded = decode(&first).expect("decoded image");

    assert_eq!(first, second);
    assert_eq!(decoded.identity(), image.identity());
    assert_eq!(decoded.source_paths(), image.source_paths());
    assert_eq!(decoded.source_hashes(), image.source_hashes());
    assert_eq!(decoded.executable(), image.executable());
}

#[test]
fn canonical_image_has_target_independent_digest() {
    let bytes = encode(&program_image()).expect("encoded image");

    assert_eq!(
        format!("{:?}", fpas_program::Digest::of(bytes)),
        "17c06539c8320bb96f6f8496263db98548496663b5f712f0473260f2e8581d62"
    );
}

#[test]
fn every_truncated_prefix_is_rejected_without_panic() {
    let bytes = encode(&program_image()).expect("encoded image");

    for length in 0..bytes.len() {
        let result = catch_unwind(AssertUnwindSafe(|| decode(&bytes[..length])));
        assert!(result.is_ok(), "decoder panicked at prefix {length}");
        assert!(
            result.expect("caught decoder").is_err(),
            "prefix {length} decoded"
        );
    }
}

#[test]
fn decoder_rejects_old_stack_format_with_versions() {
    let mut bytes = encode(&program_image()).expect("encoded image");
    bytes[8..10].copy_from_slice(&1_u16.to_le_bytes());

    assert_eq!(
        decode_error(&bytes),
        FormatError::UnsupportedVersion {
            image: 1,
            runtime: PROGRAM_FORMAT_VERSION,
        }
    );
}

#[test]
fn decoder_rejects_incompatible_bytecode_version() {
    let mut bytes = encode(&program_image()).expect("encoded image");
    bytes[10..14].copy_from_slice(&99_u32.to_le_bytes());

    assert_eq!(
        decode_error(&bytes),
        FormatError::UnsupportedBytecodeVersion {
            image: 99,
            runtime: fpas_bytecode::BYTECODE_VERSION,
        }
    );
}

#[test]
fn decoder_rejects_payload_digest_mismatch_and_trailing_bytes() {
    let mut corrupt = encode(&program_image()).expect("encoded image");
    *corrupt.last_mut().expect("payload byte") ^= 0x80;
    assert_eq!(decode_error(&corrupt), FormatError::PayloadHashMismatch);

    let mut trailing = encode(&program_image()).expect("encoded image");
    trailing.extend_from_slice(&[1, 2]);
    assert_eq!(
        decode_error(&trailing),
        FormatError::TrailingBytes {
            container: "envelope",
            count: 2,
        }
    );
}

#[test]
fn decoder_rejects_duplicate_missing_unknown_and_out_of_order_tags() {
    for tag in [1_u16, 11, 3] {
        let mut bytes = encode(&program_image()).expect("encoded image");
        let directory = payload_start(&bytes);
        bytes[directory + 4 + 16..directory + 6 + 16].copy_from_slice(&tag.to_le_bytes());
        refresh_payload_digest(&mut bytes);
        assert!(matches!(
            decode_error(&bytes),
            FormatError::SectionTag { .. }
        ));
    }
}

#[test]
fn decoder_rejects_overlapping_out_of_bounds_and_gapped_sections() {
    for offset in [0_u32, u32::MAX, 165] {
        let mut bytes = encode(&program_image()).expect("encoded image");
        let directory = payload_start(&bytes);
        bytes[directory + 8..directory + 12].copy_from_slice(&offset.to_le_bytes());
        refresh_payload_digest(&mut bytes);
        assert!(matches!(
            decode_error(&bytes),
            FormatError::SectionRange { .. }
        ));
    }
}

#[test]
fn decoder_rejects_invalid_utf8_unknown_opcode_and_noncanonical_boolean() {
    let base = encode(&program_image()).expect("encoded image");
    for (tag_index, within_section, value) in
        [(0_usize, 4_usize, 0xff_u8), (7, 0, 0xff), (1, 59, 2)]
    {
        let mut bytes = base.clone();
        let payload = payload_start(&bytes);
        let entry = payload + 4 + tag_index * 16;
        let section_offset = u32::from_le_bytes(
            bytes[entry + 4..entry + 8]
                .try_into()
                .expect("section offset"),
        ) as usize;
        bytes[payload + section_offset + within_section] = value;
        refresh_payload_digest(&mut bytes);
        assert!(decode(&bytes).is_err());
    }
}

#[test]
fn deterministic_payload_mutations_never_panic() {
    let base = encode(&program_image()).expect("encoded image");
    let payload = payload_start(&base);
    for index in (payload..base.len()).step_by(7) {
        let mut bytes = base.clone();
        bytes[index] ^= 0x5a;
        refresh_payload_digest(&mut bytes);
        assert!(catch_unwind(AssertUnwindSafe(|| decode(&bytes))).is_ok());
    }
}
