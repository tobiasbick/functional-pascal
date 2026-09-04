mod search;

use super::*;
use crate::limits::MAX_COLLECTION_LEN;
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn run_str(intrinsic: StrIntrinsic, stack: &mut Vec<Value>) -> Result<(), StdError> {
    crate::execute_test_intrinsic(Intrinsic::Str(intrinsic), stack, loc()).map(|_| ())
}

#[test]
fn length_counts_ascii_bytes_and_unicode_scalars() {
    let mut ascii = vec![Value::Str("hello".into())];
    run_str(StrIntrinsic::Length, &mut ascii).unwrap();
    assert_eq!(ascii, vec![Value::Integer(5)]);

    let mut unicode = vec![Value::Str("café".into())];
    run_str(StrIntrinsic::Length, &mut unicode).unwrap();
    assert_eq!(unicode, vec![Value::Integer(4)]);
}

#[test]
fn repeat_str_builds_requested_length() {
    let mut stack = vec![Value::Str("ab".into()), Value::Integer(3)];
    run_str(StrIntrinsic::Repeat, &mut stack).unwrap();
    assert_eq!(stack, vec![Value::Str("ababab".into())]);
}

#[test]
fn repeat_str_rejects_count_above_limit() {
    let mut stack = vec![
        Value::Str("x".into()),
        Value::Integer(MAX_COLLECTION_LEN + 1),
    ];
    let err = run_str(StrIntrinsic::Repeat, &mut stack)
        .expect_err("RepeatStr must reject a count above the collection limit");
    assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn substring_rejects_overflowing_range() {
    let mut stack = vec![
        Value::Str("ab".into()),
        Value::Integer(1),
        Value::Integer(i64::MAX),
    ];
    let err = run_str(StrIntrinsic::Substring, &mut stack)
        .expect_err("Substring must reject an overflowing range");
    assert_eq!(err.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn delete_rejects_overflowing_range() {
    let mut stack = vec![
        Value::Str("ab".into()),
        Value::Integer(1),
        Value::Integer(i64::MAX),
    ];
    let err = run_str(StrIntrinsic::Delete, &mut stack)
        .expect_err("Delete must reject an overflowing range");
    assert_eq!(err.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn from_char_returns_empty_for_non_positive_count() {
    for count in [0, -1] {
        let mut stack = vec![Value::Str("x".into()), Value::Integer(count)];
        run_str(StrIntrinsic::FromChar, &mut stack).unwrap();
        assert_eq!(stack, vec![Value::Str("".into())]);
    }
}

#[test]
fn from_char_rejects_count_above_limit() {
    let mut stack = vec![
        Value::Str("x".into()),
        Value::Integer(MAX_COLLECTION_LEN + 1),
    ];
    let err = run_str(StrIntrinsic::FromChar, &mut stack)
        .expect_err("FromChar must reject a count above the collection limit");
    assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn padding_rejects_width_above_limit() {
    let mut stack = vec![
        Value::Str("x".into()),
        Value::Integer(MAX_COLLECTION_LEN + 1),
        Value::Str(" ".into()),
    ];
    let err = run_str(StrIntrinsic::PadLeft, &mut stack)
        .expect_err("PadLeft must reject a width above the collection limit");
    assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn character_apis_accept_one_unicode_scalar() {
    let mut from_char = vec![Value::Str("😀".into()), Value::Integer(2)];
    run_str(StrIntrinsic::FromChar, &mut from_char).unwrap();
    assert_eq!(from_char, vec![Value::Str("😀😀".into())]);

    let mut set_char_at = vec![
        Value::Str("a😀c".into()),
        Value::Integer(1),
        Value::Str("ß".into()),
    ];
    run_str(StrIntrinsic::SetCharAt, &mut set_char_at).unwrap();
    assert_eq!(set_char_at, vec![Value::Str("aßc".into())]);

    let mut ord = vec![Value::Str("😀".into())];
    run_str(StrIntrinsic::Ord, &mut ord).unwrap();
    assert_eq!(ord, vec![Value::Integer(0x1F600)]);
}

#[test]
fn character_apis_reject_empty_and_multiple_scalars() {
    for (intrinsic, stack) in [
        (
            StrIntrinsic::FromChar,
            vec![Value::Str("".into()), Value::Integer(1)],
        ),
        (
            StrIntrinsic::FromChar,
            vec![Value::Str("ab".into()), Value::Integer(1)],
        ),
        (
            StrIntrinsic::SetCharAt,
            vec![
                Value::Str("abc".into()),
                Value::Integer(1),
                Value::Str("".into()),
            ],
        ),
        (
            StrIntrinsic::SetCharAt,
            vec![
                Value::Str("abc".into()),
                Value::Integer(1),
                Value::Str("ab".into()),
            ],
        ),
        (StrIntrinsic::Ord, vec![Value::Str("".into())]),
        (StrIntrinsic::Ord, vec![Value::Str("ab".into())]),
    ] {
        let mut stack = stack;
        let err = run_str(intrinsic, &mut stack)
            .expect_err("character APIs must reject invalid scalar payloads");
        assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }
}
