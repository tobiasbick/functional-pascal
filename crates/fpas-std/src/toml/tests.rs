//! Std.Toml parsing, conversion, and numeric boundaries.

use super::*;

#[test]
fn parsed_nonfinite_values_are_errors_before_they_enter_fpas_aggregates() {
    for text in [
        "value = nan",
        "value = inf",
        "value = -inf",
        "items = [1.0, +inf]",
        "[nested]\nvalue = nan",
    ] {
        let mut stack = vec![Value::Str(text.into())];
        crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
            .expect("parse returns a Result");
        let [Value::ResultError(message)] = stack.as_slice() else {
            panic!("nonfinite input must not produce an FPAS value: {text}");
        };
        assert!(matches!(message.as_ref(), Value::Str(text) if text.contains("not finite")));
    }
}

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn call() -> IntrinsicCall<'static> {
    IntrinsicCall::new(&[], &crate::intrinsics::TEST_AGGREGATES)
}

fn test_variant(variant: &str, fields: Vec<Value>) -> Value {
    toml_variant(&call(), variant, fields, loc()).expect("test aggregate layout")
}

fn test_toml_to_fpas_at_depth(value: TomlValue, depth: usize) -> Result<Value, StdError> {
    toml_to_fpas_at_depth(&call(), value, depth, loc())
}

#[test]
fn parse_preserves_all_toml_value_kinds() {
    let text = r#"
title = "TOML"
integer = 42
float = 1.5
boolean = true
when = 1979-05-27T07:32:00Z
items = ["a", 2]
[nested]
value = "ok"
"#;
    let mut stack = vec![Value::Str(text.into())];

    crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
        .unwrap();

    assert!(matches!(stack.as_slice(), [Value::ResultOk(_)]));
    let Value::ResultOk(value) = &stack[0] else {
        panic!("expected a parsed TOML value");
    };
    let Value::Enum(value) = value.as_ref() else {
        panic!("expected a TomlValue enum");
    };
    assert_eq!(value.body().layout.variant, "Table");
}

#[test]
fn parse_returns_error_for_invalid_toml() {
    let mut stack = vec![Value::Str("answer = [1,".into())];

    crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
        .unwrap();

    assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
}

#[test]
fn toml_to_fpas_accepts_container_at_depth_limit() {
    let value = TomlValue::Array(vec![TomlValue::String("ok".into())]);

    assert!(test_toml_to_fpas_at_depth(value, MAX_TOML_DEPTH - 1).is_ok());
}

#[test]
fn toml_to_fpas_rejects_container_child_beyond_depth_limit() {
    let value = TomlValue::Array(vec![TomlValue::String("too deep".into())]);

    assert!(test_toml_to_fpas_at_depth(value, MAX_TOML_DEPTH).is_err());
}

#[test]
fn fpas_to_toml_accepts_container_at_depth_limit() {
    let value = test_variant(
        "ArrayValue",
        vec![Value::Array(
            vec![test_variant("String", vec![Value::Str("ok".into())])].into(),
        )],
    );

    assert!(fpas_to_toml_at_depth(value, loc(), MAX_TOML_DEPTH - 1).is_ok());
}

#[test]
fn fpas_to_toml_rejects_container_child_beyond_depth_limit() {
    let value = test_variant(
        "ArrayValue",
        vec![Value::Array(
            vec![test_variant("String", vec![Value::Str("too deep".into())])].into(),
        )],
    );

    let error = fpas_to_toml_at_depth(value, loc(), MAX_TOML_DEPTH)
        .expect_err("TOML conversion must enforce its nesting limit");

    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn stringify_rejects_invalid_datetime() {
    let value = test_variant("Datetime", vec![Value::Str("not-a-date".into())]);

    let error =
        fpas_to_toml(value, loc()).expect_err("TOML conversion must reject an invalid datetime");

    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert!(error.message.contains("Datetime is invalid"));
}

#[test]
fn stringify_round_trips_array_of_tables() {
    let mut stack = vec![Value::Str(
        "[[products]]\nname = \"Hammer\"\n[[products]]\nname = \"Nail\"\n".into(),
    )];
    crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
        .unwrap();
    let Value::ResultOk(value) = stack.pop().expect("parse result") else {
        panic!("expected a parsed TOML value");
    };
    let mut stringify_stack = vec![value.into_inner()];

    crate::execute_test_intrinsic(
        Intrinsic::Toml(TomlIntrinsic::Stringify),
        &mut stringify_stack,
        loc(),
    )
    .unwrap();

    let Value::Str(text) = stringify_stack.pop().expect("stringified TOML") else {
        panic!("expected TOML text");
    };
    assert!(text.contains("[[products]]"));
}
