//! Binary `.fpascu` envelope tests.

#![allow(
    clippy::expect_used,
    reason = "binary format fixtures use expect for compact round-trip assertions"
)]

use fpas_unit::{
    CompiledUnit, DependencyIdentity, Digest, FormatError, UnitIdentity, decode, encode,
};

fn compiled_unit() -> CompiledUnit {
    let interface = vec![1, 2, 3, 4];
    let object = vec![10, 20, 30];
    CompiledUnit {
        identity: UnitIdentity {
            unit_name: "demo.core".to_string(),
            source_hash: Digest::of(b"unit Demo.Core;"),
            interface_hash: Digest::of(&interface),
            object_hash: Digest::of(&object),
            compiler_version: "0.0.1-test".to_string(),
            bytecode_version: 7,
            options_hash: Digest::of(b"default-options"),
            dependencies: vec![DependencyIdentity {
                unit_name: "demo.base".to_string(),
                interface_hash: Digest::of(b"base-interface"),
            }],
        },
        interface,
        object,
    }
}

#[test]
fn format_round_trip_preserves_complete_compiled_unit() {
    let unit = compiled_unit();

    let bytes = encode(&unit).expect("unit must encode");
    let decoded = decode(&bytes).expect("unit must decode");

    assert_eq!(decoded, unit);
}

#[test]
fn format_encoding_is_deterministic() {
    let unit = compiled_unit();

    let first = encode(&unit).expect("first encoding");
    let second = encode(&unit).expect("second encoding");

    assert_eq!(first, second);
}

#[test]
fn decoder_rejects_invalid_magic() {
    let mut bytes = encode(&compiled_unit()).expect("encoding");
    bytes[0] ^= 0xff;

    assert_eq!(decode(&bytes), Err(FormatError::InvalidMagic));
}

#[test]
fn decoder_reports_unsupported_format_version() {
    let mut bytes = encode(&compiled_unit()).expect("encoding");
    bytes[8..10].copy_from_slice(&99_u16.to_le_bytes());

    assert_eq!(decode(&bytes), Err(FormatError::UnsupportedVersion(99)));
}

#[test]
fn decoder_rejects_every_truncated_prefix_without_panicking() {
    let bytes = encode(&compiled_unit()).expect("encoding");

    for length in 0..bytes.len() {
        assert!(
            decode(&bytes[..length]).is_err(),
            "prefix of {length} bytes must fail"
        );
    }
}

#[test]
fn decoder_rejects_trailing_bytes() {
    let mut bytes = encode(&compiled_unit()).expect("encoding");
    bytes.extend_from_slice(&[1, 2]);

    assert_eq!(decode(&bytes), Err(FormatError::TrailingBytes(2)));
}

#[test]
fn decoder_rejects_interface_payload_hash_mismatch() {
    let mut unit = compiled_unit();
    unit.object.clear();
    unit.identity.object_hash = Digest::of(&unit.object);
    let mut bytes = encode(&unit).expect("encoding");
    let interface_position = bytes
        .windows(unit.interface.len())
        .position(|window| window == unit.interface)
        .expect("interface payload must be present");
    bytes[interface_position] ^= 0xff;

    assert_eq!(decode(&bytes), Err(FormatError::InterfaceHashMismatch));
}

#[test]
fn decoder_rejects_object_payload_hash_mismatch() {
    let unit = compiled_unit();
    let mut bytes = encode(&unit).expect("encoding");
    let object_position = bytes
        .windows(unit.object.len())
        .rposition(|window| window == unit.object)
        .expect("object payload must be present");
    bytes[object_position] ^= 0xff;

    assert_eq!(decode(&bytes), Err(FormatError::ObjectHashMismatch));
}

#[test]
fn encoder_rejects_inconsistent_interface_hash() {
    let mut unit = compiled_unit();
    unit.identity.interface_hash = Digest::of(b"different");

    assert_eq!(encode(&unit), Err(FormatError::InterfaceHashMismatch));
}

#[test]
fn encoder_rejects_oversized_identity_string() {
    let mut unit = compiled_unit();
    unit.identity.unit_name = "x".repeat(1024 * 1024 + 1);

    assert!(matches!(
        encode(&unit),
        Err(FormatError::LimitExceeded {
            field: "unit_name",
            ..
        })
    ));
}
