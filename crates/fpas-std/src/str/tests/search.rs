//! Search, numeric predicate, and character-index boundary behavior.

use super::{loc, run_str};
use crate::execute_test_intrinsic;
use fpas_bytecode::{Intrinsic, StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

#[test]
fn search_preserves_empty_unicode_and_missing_matches() {
    for (text, needle, contains, starts, ends, first, last) in [
        ("", "", true, true, true, 0, 0),
        ("", "é", false, false, false, -1, -1),
        ("aé😀é", "", true, true, true, 0, 4),
        ("aé😀é", "é", true, false, true, 1, 3),
        ("aé😀é", "aé", true, true, false, 0, 0),
        ("aé😀é", "😀", true, false, false, 2, 2),
        ("aé😀é", "e", false, false, false, -1, -1),
        ("é", "éé", false, false, false, -1, -1),
    ] {
        for (intrinsic, expected) in [
            (StrIntrinsic::Contains, Value::Boolean(contains)),
            (StrIntrinsic::StartsWith, Value::Boolean(starts)),
            (StrIntrinsic::EndsWith, Value::Boolean(ends)),
            (StrIntrinsic::IndexOf, Value::Integer(first)),
            (StrIntrinsic::LastIndexOf, Value::Integer(last)),
        ] {
            let mut stack = vec![Value::Str(text.into()), Value::Str(needle.into())];
            run_str(intrinsic, &mut stack).unwrap();
            assert_eq!(stack, [expected], "{intrinsic:?} ({text:?}, {needle:?})");
        }
    }
}

#[test]
fn character_queries_preserve_scalar_indices_and_reject_invalid_indices() {
    for (index, expected) in [(0, "a"), (1, "é"), (2, "😀")] {
        let mut stack = vec![Value::Str("aé😀".into()), Value::Integer(index)];
        run_str(StrIntrinsic::CharAt, &mut stack).unwrap();
        assert_eq!(stack, [Value::Str(expected.into())]);
    }
    for (text, index) in [("", 0), ("aé😀", -1), ("aé😀", 3), ("aé😀", i64::MAX)] {
        let mut stack = vec![Value::Str(text.into()), Value::Integer(index)];
        let error = run_str(StrIntrinsic::CharAt, &mut stack).expect_err("invalid character index");
        assert_eq!(error.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
    }
}

#[test]
fn numeric_predicate_preserves_valid_and_invalid_text() {
    for (text, expected) in [
        ("", false),
        ("  -1_024  ", true),
        ("1.5e+2", true),
        ("1__2", false),
        ("😀", false),
        ("NaN", false),
    ] {
        let mut stack = vec![Value::Str(text.into())];
        run_str(StrIntrinsic::IsNumeric, &mut stack).unwrap();
        assert_eq!(stack, [Value::Boolean(expected)], "{text:?}");
    }
}

#[test]
fn search_queries_reject_missing_or_non_string_arguments() {
    for intrinsic in [
        StrIntrinsic::Contains,
        StrIntrinsic::StartsWith,
        StrIntrinsic::EndsWith,
        StrIntrinsic::IndexOf,
        StrIntrinsic::LastIndexOf,
    ] {
        for mut arguments in [vec![Value::Str("text".into())], vec![]] {
            let error = execute_test_intrinsic(Intrinsic::Str(intrinsic), &mut arguments, loc())
                .expect_err("missing argument");
            assert_eq!(error.code, RUNTIME_INTRINSIC_STACK_STATE_ERROR);
        }
        for mut arguments in [
            vec![Value::Integer(1), Value::Str("x".into())],
            vec![Value::Str("x".into()), Value::Integer(1)],
        ] {
            let error = run_str(intrinsic, &mut arguments).expect_err("wrong argument type");
            assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
        }
    }
}
