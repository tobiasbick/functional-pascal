use super::*;

const TEST_LIMITS: ResourceLimits = ResourceLimits {
    instructions: 2,
    locations: 2,
    functions: 2,
    constants: 2,
    string_bytes: 32,
};

fn decode(bytes: &[u8], limits: ResourceLimits) -> Result<EncodedChunk, serde_json::Error> {
    decode_encoded_chunk_with_limits(bytes, 0, limits)
}

#[test]
fn instruction_limit_accepts_boundary_and_rejects_next_item() {
    let exact = br#"{"code":["Halt","Halt"],"constants":[],"locations":[],"functions":{}}"#;
    let over = br#"{"code":["Halt","Halt","Halt"],"constants":[],"locations":[],"functions":{}}"#;

    assert!(decode(exact, TEST_LIMITS).is_ok());
    assert!(decode(over, TEST_LIMITS).is_err());
}

#[test]
fn location_limit_accepts_boundary_and_rejects_next_item() {
    let location = r#"{"line":1,"column":1,"source_id":0}"#;
    let exact = format!(
        r#"{{"code":[],"constants":[],"locations":[{location},{location}],"functions":{{}}}}"#
    );
    let over = format!(
        r#"{{"code":[],"constants":[],"locations":[{location},{location},{location}],"functions":{{}}}}"#
    );

    assert!(decode(exact.as_bytes(), TEST_LIMITS).is_ok());
    assert!(decode(over.as_bytes(), TEST_LIMITS).is_err());
}

#[test]
fn constant_limit_accepts_boundary_and_rejects_next_item() {
    let exact =
        br#"{"code":[],"constants":[{"Integer":1},{"Integer":2}],"locations":[],"functions":{}}"#;
    let over = br#"{"code":[],"constants":[{"Integer":1},{"Integer":2},{"Integer":3}],"locations":[],"functions":{}}"#;

    assert!(decode(exact, TEST_LIMITS).is_ok());
    assert!(decode(over, TEST_LIMITS).is_err());
}

#[test]
fn function_limit_accepts_boundary_and_rejects_next_item() {
    let exact = br#"{"code":[],"constants":[],"locations":[],"functions":{"a":{"code_start":0,"arity":0},"b":{"code_start":0,"arity":0}}}"#;
    let over = br#"{"code":[],"constants":[],"locations":[],"functions":{"a":{"code_start":0,"arity":0},"b":{"code_start":0,"arity":0},"c":{"code_start":0,"arity":0}}}"#;

    assert!(decode(exact, TEST_LIMITS).is_ok());
    assert!(decode(over, TEST_LIMITS).is_err());
}

#[test]
fn cumulative_string_limit_accepts_boundary_and_rejects_next_byte() {
    let limits = ResourceLimits {
        string_bytes: 2,
        ..TEST_LIMITS
    };
    let exact = br#"{"code":[],"constants":[{"String":"ab"}],"locations":[],"functions":{}}"#;
    let over = br#"{"code":[],"constants":[{"String":"abc"}],"locations":[],"functions":{}}"#;

    assert!(decode(exact, limits).is_ok());
    assert!(decode(over, limits).is_err());
}

#[test]
fn production_resource_limits_are_inclusive() {
    for (field, maximum) in [
        ("instructions", MAX_INSTRUCTIONS),
        ("locations", MAX_LOCATIONS),
        ("functions", MAX_FUNCTIONS),
        ("constants", MAX_CONSTANTS),
        ("strings", MAX_TOTAL_STRING_BYTES),
    ] {
        assert!(check_resource_size(field, maximum, maximum).is_ok());
        assert_eq!(
            check_resource_size(field, maximum + 1, maximum),
            Err(ImageError::ResourceLimit {
                field,
                size: maximum + 1,
                maximum,
            })
        );
    }
}
