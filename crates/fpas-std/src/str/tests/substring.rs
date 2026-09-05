//! Unicode scalar ranges, boundary errors, and intrinsic argument validation.

use super::run_str;
use fpas_bytecode::{StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

#[test]
fn substring_matches_scalar_slices_including_empty_and_full_ranges() {
    for text in ["", "abc", "aé😀界", "e\u{301}\0z", "😀😀"] {
        let chars: Vec<_> = text.chars().collect();
        for start in 0..=chars.len() {
            for len in 0..=chars.len() - start {
                let mut args = vec![
                    Value::Str(text.into()),
                    Value::Integer(start as i64),
                    Value::Integer(len as i64),
                ];
                let expected: String = chars[start..start + len].iter().collect();
                run_str(StrIntrinsic::Substring, &mut args).unwrap();
                assert_eq!(
                    args,
                    [Value::Str(expected.into())],
                    "{text:?}: {start}+{len}"
                );
            }
        }
    }
}

#[test]
fn substring_rejects_negative_outside_and_overflowing_ranges() {
    for text in ["", "abc", "aé😀界"] {
        let n = text.chars().count() as i64;
        for (start, len) in [
            (-1, 0),
            (0, -1),
            (n + 1, 0),
            (n, 1),
            (0, n + 1),
            (i64::MIN, 0),
            (i64::MAX, 0),
            (1, i64::MAX),
            (0, i64::MAX),
        ] {
            let mut args = vec![
                Value::Str(text.into()),
                Value::Integer(start),
                Value::Integer(len),
            ];
            let error = run_str(StrIntrinsic::Substring, &mut args).expect_err("invalid range");
            assert_eq!(error.code, RUNTIME_STRING_INDEX_OUT_OF_BOUNDS);
            assert_eq!(
                error.message,
                format!("Substring out of range (len={n}, start={start}, len_param={len})")
            );
        }
    }
}

#[test]
fn substring_rejects_missing_and_wrong_type_arguments() {
    for mut args in [
        vec![],
        vec![Value::Integer(1)],
        vec![Value::Integer(0), Value::Integer(1)],
    ] {
        let error = run_str(StrIntrinsic::Substring, &mut args).expect_err("missing argument");
        assert_eq!(error.code, RUNTIME_INTRINSIC_STACK_STATE_ERROR);
    }
    for index in 0..3 {
        let mut args = vec![
            Value::Str("abc".into()),
            Value::Integer(0),
            Value::Integer(1),
        ];
        args[index] = Value::Boolean(false);
        let error = run_str(StrIntrinsic::Substring, &mut args).expect_err("wrong argument type");
        assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }
}

#[test]
fn substring_preserves_shared_source_and_cached_character_count() {
    let source = fpas_bytecode::SharedStr::from("aé😀界");
    for (start, len, expected) in [(0, 4, "aé😀界"), (1, 2, "é😀"), (4, 0, "")] {
        let mut args = vec![
            Value::Str(source.clone()),
            Value::Integer(start),
            Value::Integer(len),
        ];
        run_str(StrIntrinsic::Substring, &mut args).unwrap();
        let Value::Str(result) = &args[0] else {
            panic!("expected string")
        };
        assert_eq!(result.as_ref(), expected);
        assert_eq!(result.char_len(), len as usize);
        assert_eq!(source.as_ref(), "aé😀界");
    }
}
