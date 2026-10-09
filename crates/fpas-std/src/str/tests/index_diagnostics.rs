//! Character and insertion bounds hints use the native string call forms.
//!
//! **Documentation:** `docs/pascal/language/types/string/format-chars.md`,
//! `docs/pascal/language/types/string/edit.md`.

use super::run_str;
use fpas_bytecode::{StrIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_STRING_INDEX_OUT_OF_BOUNDS;

fn character_arguments(intrinsic: StrIntrinsic, text: &str, index: i64) -> Vec<Value> {
    let mut arguments = vec![Value::Str(text.into()), Value::Integer(index)];
    if intrinsic == StrIntrinsic::SetCharAt {
        arguments.push(Value::Str("z".into()));
    }
    arguments
}

#[test]
fn nonempty_character_bounds_hints_use_native_length() {
    for intrinsic in [StrIntrinsic::CharAt, StrIntrinsic::SetCharAt] {
        for index in [-1, 2, i64::MIN, i64::MAX] {
            let mut arguments = character_arguments(intrinsic, "é😀", index);
            let error = run_str(intrinsic, &mut arguments).expect_err("invalid scalar index");
            assert_eq!(error.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
            assert_eq!(
                error.message,
                format!("{intrinsic:?} index {index} out of range (length 2)")
            );
            assert_eq!(
                error.help.as_deref(),
                Some("Ensure the index is within 0..S.Length()-1.")
            );
        }
    }
}

#[test]
fn empty_character_bounds_hints_report_no_valid_index() {
    for intrinsic in [StrIntrinsic::CharAt, StrIntrinsic::SetCharAt] {
        for index in [-1, 0, 1, i64::MIN, i64::MAX] {
            let mut arguments = character_arguments(intrinsic, "", index);
            let error = run_str(intrinsic, &mut arguments).expect_err("empty character source");
            assert_eq!(error.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
            assert_eq!(
                error.help.as_deref(),
                Some(
                    "An empty string has no valid character index. Check S.IsEmpty() before calling this operation."
                )
            );
        }
    }
}

#[test]
fn insertion_bounds_hints_include_the_end_position_for_empty_and_unicode_strings() {
    for (text, length) in [("", 0), ("é😀", 2)] {
        for index in [-1, length + 1, i64::MIN, i64::MAX] {
            let mut arguments = vec![
                Value::Str(text.into()),
                Value::Integer(index),
                Value::Str("z".into()),
            ];
            let error = run_str(StrIntrinsic::Insert, &mut arguments)
                .expect_err("invalid scalar insertion position");
            assert_eq!(error.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
            assert_eq!(
                error.message,
                format!("Insert index {index} out of range (length {length})")
            );
            assert_eq!(
                error.help.as_deref(),
                Some("Ensure the index is within 0..S.Length().")
            );
        }
    }
}

#[test]
fn character_operations_keep_valid_first_and_last_scalar_indices() {
    for (text, index, character, replaced) in [
        ("a", 0, "a", "z"),
        ("é😀", 0, "é", "z😀"),
        ("é😀", 1, "😀", "éz"),
    ] {
        let mut arguments = character_arguments(StrIntrinsic::CharAt, text, index);
        run_str(StrIntrinsic::CharAt, &mut arguments).expect("valid character index");
        assert_eq!(arguments, [Value::Str(character.into())]);
        let mut arguments = character_arguments(StrIntrinsic::SetCharAt, text, index);
        run_str(StrIntrinsic::SetCharAt, &mut arguments).expect("valid replacement index");
        assert_eq!(arguments, [Value::Str(replaced.into())]);
    }
}

#[test]
fn insertion_keeps_valid_start_and_end_positions_including_empty_strings() {
    for (text, index, expected) in [("", 0, "z"), ("é😀", 0, "zé😀"), ("é😀", 2, "é😀z")]
    {
        let mut arguments = vec![
            Value::Str(text.into()),
            Value::Integer(index),
            Value::Str("z".into()),
        ];
        run_str(StrIntrinsic::Insert, &mut arguments).expect("valid insertion position");
        assert_eq!(arguments, [Value::Str(expected.into())]);
    }
}
