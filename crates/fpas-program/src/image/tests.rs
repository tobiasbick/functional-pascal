use super::*;
use crate::Digest;

fn identity() -> ProgramIdentity {
    ProgramIdentity {
        compiler_version: "test-compiler".to_string(),
        bytecode_version: fpas_bytecode::BYTECODE_VERSION,
        source_hash: Digest::of(b"source"),
        options_hash: Digest::of(b"options"),
        units: Vec::new(),
    }
}

fn decode(bytes: &[u8]) -> Result<ProgramImage, ImageError> {
    decode_payload(identity(), vec!["main.fpas".to_string()], bytes, 0)
}

#[test]
fn payload_decoder_rejects_unknown_opcode_tag() {
    let payload = br#"{"code":["Unknown"],"constants":[],"locations":[],"functions":{}}"#;

    assert!(matches!(decode(payload), Err(ImageError::PayloadDecode(_))));
}

#[test]
fn payload_decoder_rejects_zero_based_location() {
    let payload = br#"{"code":["Halt"],"constants":[],"locations":[{"line":0,"column":1,"source_id":0}],"functions":{}}"#;

    assert_eq!(
        decode(payload).err(),
        Some(ImageError::InvalidLocation {
            instruction: 0,
            line: 0,
            column: 1,
        })
    );
}

#[test]
fn payload_decoder_rejects_duplicate_constants() {
    let payload = br#"{"code":["Halt"],"constants":[{"Integer":1},{"Integer":1}],"locations":[{"line":1,"column":1,"source_id":0}],"functions":{}}"#;

    assert_eq!(
        decode(payload).err(),
        Some(ImageError::DuplicateConstant {
            index: 1,
            existing: 0,
        })
    );
}

#[test]
fn payload_decoder_rejects_unknown_chunk_field() {
    let payload = br#"{"code":["Halt"],"constants":[],"locations":[{"line":1,"column":1,"source_id":0}],"functions":{},"extra":true}"#;

    assert!(matches!(decode(payload), Err(ImageError::PayloadDecode(_))));
}

#[test]
fn payload_decoder_rejects_unknown_location_field() {
    let payload = br#"{"code":["Halt"],"constants":[],"locations":[{"line":1,"column":1,"source_id":0,"extra":true}],"functions":{}}"#;

    assert!(matches!(decode(payload), Err(ImageError::PayloadDecode(_))));
}

#[test]
fn payload_decoder_rejects_unknown_function_field() {
    let payload = br#"{"code":["Halt"],"constants":[],"locations":[{"line":1,"column":1,"source_id":0}],"functions":{"demo":{"code_start":0,"arity":0,"extra":true}}}"#;

    assert!(matches!(decode(payload), Err(ImageError::PayloadDecode(_))));
}
