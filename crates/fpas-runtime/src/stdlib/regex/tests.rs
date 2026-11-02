use super::*;
use crate::{
    Intrinsic,
    stdlib::{intrinsics::TEST_AGGREGATES, run_intrinsic_borrowed},
};

fn invoke(operation: Op, arguments: &[Value]) -> Value {
    run_intrinsic_borrowed(
        Intrinsic::Regex(operation),
        arguments,
        SourceLocation::new(1, 1),
        &TEST_AGGREGATES,
    )
    .unwrap()
    .unwrap()
}
fn compiled(expression: &str) -> Value {
    let options = invoke(Op::DefaultOptions, &[]);
    let Value::ResultOk(value) = invoke(Op::Compile, &[Value::Str(expression.into()), options])
    else {
        panic!("valid pattern");
    };
    value.as_ref().clone()
}
fn error_variant(value: Value) -> String {
    let Value::ResultError(error) = value else {
        panic!("expected typed error");
    };
    let Value::Enum(error) = error.as_ref() else {
        panic!("nominal regex error");
    };
    assert_eq!(error.body().layout.type_name, "Std.Text.Regex.RegexError");
    error.body().layout.variant.clone()
}

#[test]
fn bad_syntax_ranges_and_sizes_are_typed_errors() {
    assert_eq!(
        error_variant(invoke(
            Op::Compile,
            &[Value::Str("(".into()), invoke(Op::DefaultOptions, &[])]
        )),
        "InvalidPattern"
    );
    assert_eq!(
        error_variant(invoke(
            Op::Compile,
            &[
                Value::Str("a".repeat(MAX_PATTERN + 1).into()),
                invoke(Op::DefaultOptions, &[])
            ]
        )),
        "LimitExceeded"
    );
    let pattern = compiled("a");
    for start in [-1, 3, i64::MAX] {
        assert_eq!(
            error_variant(invoke(
                Op::Find,
                &[
                    pattern.clone(),
                    Value::Str("éa".into()),
                    Value::Integer(start)
                ]
            )),
            "InvalidStart"
        );
    }
    assert_eq!(
        error_variant(invoke(
            Op::Split,
            &[pattern.clone(), Value::Str("a".into()), Value::Integer(-1)]
        )),
        "InvalidLimit"
    );
    assert_eq!(
        error_variant(invoke(
            Op::IsMatch,
            &[pattern.clone(), Value::Str("a".repeat(MAX_TEXT + 1).into())]
        )),
        "LimitExceeded"
    );
    assert_eq!(
        error_variant(invoke(
            Op::FindAll,
            &[
                compiled(""),
                Value::Str("x".repeat(MAX_ITEMS).into()),
                Value::Integer(0)
            ]
        )),
        "LimitExceeded"
    );
    assert_eq!(
        error_variant(invoke(
            Op::Replace,
            &[
                pattern,
                Value::Str("a".repeat(1024).into()),
                Value::Str("x".repeat(MAX_TEXT / 512).into()),
                Value::Integer(0)
            ]
        )),
        "LimitExceeded"
    );
}

#[test]
fn compiled_storage_is_shared_and_released_with_last_value() {
    let original = compiled("[0-9]+");
    let Value::Regex(inner) = &original else {
        panic!("compiled storage");
    };
    let weak = Arc::downgrade(inner);
    let copied = original.clone();
    assert!(crate::summarized_transferable(std::slice::from_ref(
        &copied
    )));
    assert_eq!(original, copied);
    drop(original);
    assert!(weak.upgrade().is_some());
    drop(copied);
    assert!(weak.upgrade().is_none());
}

#[test]
fn malformed_native_arguments_use_vm_diagnostics() {
    assert!(
        run_intrinsic_borrowed(
            Intrinsic::Regex(Op::IsMatch),
            &[Value::Integer(7), Value::Str("x".into())],
            SourceLocation::new(1, 1),
            &TEST_AGGREGATES
        )
        .is_err()
    );
}

#[test]
fn empty_and_unicode_match_iteration_agrees_with_the_engine() {
    for expression in ["", "a?", ".*?", ".*", "a|", "\\b", "\\p{L}+", "(a)?()"] {
        let native = compiled(expression);
        let engine = regex::Regex::new(expression).unwrap();
        for text in ["", "a", "aa", "é🙂a", "ba a"] {
            let Value::ResultOk(found) = invoke(
                Op::FindAll,
                &[native.clone(), Value::Str(text.into()), Value::Integer(0)],
            ) else {
                panic!("bounded search");
            };
            let Value::Array(found) = found.as_ref() else {
                panic!("match array");
            };
            let expected = engine
                .find_iter(text)
                .map(|span| (text[..span.start()].chars().count(), span.as_str()))
                .collect::<Vec<_>>();
            assert_eq!(found.len(), expected.len(), "{expression:?} {text:?}");
            for (actual, (start, expected_text)) in found.iter().zip(expected) {
                let Value::Record(actual) = actual else {
                    panic!("match record");
                };
                let Value::Record(span) = &actual.body().values[0] else {
                    panic!("span record");
                };
                assert_eq!(span.body().values[0], Value::Integer(start as i64));
                assert_eq!(span.body().values[2], Value::Str(expected_text.into()));
            }
        }
    }
}
