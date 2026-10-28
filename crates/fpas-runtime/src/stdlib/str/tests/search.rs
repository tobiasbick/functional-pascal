//! Search, numeric predicate, and character-index boundary behavior.

use super::{loc, run_str};
use crate::stdlib::execute_test_intrinsic;
use crate::{Intrinsic, StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

#[test]
fn search_preserves_empty_unicode_and_missing_matches() {
    for (text, needle, starts, ends, first, last) in [
        ("", "", true, true, Some(0), Some(0)),
        ("", "é", false, false, None, None),
        ("aé😀é", "", true, true, Some(0), Some(4)),
        ("aé😀é", "é", false, true, Some(1), Some(3)),
        ("aé😀é", "aé", true, false, Some(0), Some(0)),
        ("aé😀é", "😀", false, false, Some(2), Some(2)),
        ("aé😀é", "e", false, false, None, None),
        ("é", "éé", false, false, None, None),
    ] {
        let optional_index = |index: Option<i64>| {
            index.map_or(Value::OptionNone, |index| {
                Value::option_some(Value::Integer(index))
            })
        };
        for (intrinsic, expected) in [
            (StrIntrinsic::StartsWith, Value::Boolean(starts)),
            (StrIntrinsic::EndsWith, Value::Boolean(ends)),
            (StrIntrinsic::IndexOf, optional_index(first)),
            (StrIntrinsic::LastIndexOf, optional_index(last)),
        ] {
            let mut stack = vec![Value::Str(text.into()), Value::Str(needle.into())];
            run_str(intrinsic, &mut stack).unwrap();
            assert_eq!(stack, [expected], "{intrinsic:?} ({text:?}, {needle:?})");
        }
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
