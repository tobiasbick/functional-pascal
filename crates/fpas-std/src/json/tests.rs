use super::*;
use crate::limits::MAX_JSON_DEPTH;

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn call() -> IntrinsicCall<'static> {
    IntrinsicCall::new(&[], &crate::intrinsics::TEST_AGGREGATES)
}

fn test_variant(variant: &str, fields: Vec<Value>) -> Value {
    json_variant(&call(), variant, fields, loc()).expect("test aggregate layout")
}

fn test_json_to_fpas_at_depth(value: JsonValue, depth: usize) -> Result<Value, StdError> {
    json_to_fpas_at_depth(&call(), value, depth, loc())
}

fn nested_array_json(levels: usize) -> String {
    let mut text = String::from("null");
    for _ in 0..levels {
        text = format!("[{text}]");
    }
    text
}

#[test]
fn parse_accepts_shallow_json() {
    let mut stack = vec![Value::Str("null".into())];
    crate::execute_test_intrinsic(Intrinsic::Json(JsonIntrinsic::Parse), &mut stack, loc())
        .unwrap();
    assert!(matches!(stack.as_slice(), [Value::ResultOk(_)]));
}

#[test]
fn parsed_json_null_uses_the_null_value_variant() {
    let value = test_json_to_fpas_at_depth(JsonValue::Null, 1).expect("JSON null converts");
    let Value::Enum(value) = value else {
        panic!("JSON null must produce an enum value")
    };
    assert_eq!(value.body().layout.variant, "NullValue");
    assert!(value.body().values.is_empty());
}

#[test]
fn null_value_serializes_as_json_null() {
    let value = test_variant("NullValue", vec![]);
    let json = fpas_to_json(value, loc()).expect("NullValue converts");
    assert_eq!(json.to_string(), "null");
}

#[test]
fn json_to_fpas_accepts_container_at_depth_limit() {
    let json = JsonValue::Array(vec![JsonValue::Null]);
    assert!(test_json_to_fpas_at_depth(json, MAX_JSON_DEPTH - 1).is_ok());
}

#[test]
fn json_to_fpas_rejects_container_child_beyond_depth_limit() {
    let json = JsonValue::Array(vec![JsonValue::Null]);
    assert!(test_json_to_fpas_at_depth(json, MAX_JSON_DEPTH).is_err());
}

#[test]
fn parse_rejects_json_above_max_depth() {
    let mut stack = vec![Value::Str(nested_array_json(MAX_JSON_DEPTH).into())];
    crate::execute_test_intrinsic(Intrinsic::Json(JsonIntrinsic::Parse), &mut stack, loc())
        .unwrap();
    assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
}

#[test]
fn fpas_to_json_accepts_container_at_depth_limit() {
    let value = test_variant(
        "ArrayValue",
        vec![Value::Array(vec![test_variant("NullValue", vec![])].into())],
    );
    assert!(fpas_to_json_at_depth(value, loc(), MAX_JSON_DEPTH - 1).is_ok());
}

#[test]
fn fpas_to_json_rejects_container_child_beyond_depth_limit() {
    let value = test_variant(
        "ArrayValue",
        vec![Value::Array(vec![test_variant("NullValue", vec![])].into())],
    );
    let err = fpas_to_json_at_depth(value, loc(), MAX_JSON_DEPTH)
        .expect_err("JSON conversion must enforce its nesting limit");
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

fn stringified_number(value: f64) -> String {
    let json = fpas_to_json_at_depth(test_variant("Number", vec![Value::Real(value)]), loc(), 0)
        .expect("finite number converts");
    serde_json::to_string(&json).expect("number serializes")
}

#[test]
fn stringify_keeps_dictionary_insertion_order() {
    let fields = vec![
        (
            Value::Str("zeta".into()),
            test_variant("Number", vec![Value::Real(1.0)]),
        ),
        (
            Value::Str("alpha".into()),
            test_variant("Number", vec![Value::Real(2.0)]),
        ),
    ];
    let object = test_variant("Object", vec![Value::dict(fields)]);
    let json = fpas_to_json_at_depth(object, loc(), 0).expect("object converts");
    assert_eq!(
        serde_json::to_string(&json).unwrap(),
        r#"{"zeta":1,"alpha":2}"#
    );
}

#[test]
fn parse_keeps_document_member_order() {
    let text = r#"{"zeta":1,"alpha":{"y":2,"b":3},"mid":[{"z":4,"a":5}]}"#;
    let parsed = parse::parse(text).expect("valid JSON");
    let value = test_json_to_fpas_at_depth(parsed, 1).expect("converts to FPAS");
    let json = fpas_to_json_at_depth(value, loc(), 1).expect("converts back");
    assert_eq!(serde_json::to_string(&json).unwrap(), text);
}

#[test]
fn integral_numbers_stringify_without_a_fraction() {
    assert_eq!(stringified_number(2.0), "2");
    assert_eq!(stringified_number(-3.0), "-3");
    assert_eq!(stringified_number(0.0), "0");
    assert_eq!(
        stringified_number(9_007_199_254_740_992.0),
        "9007199254740992"
    );
}

#[test]
fn other_numbers_keep_their_real_form() {
    assert_eq!(stringified_number(1.5), "1.5");
    assert_eq!(stringified_number(-0.0), "-0.0");
    assert_eq!(
        stringified_number(18_014_398_509_481_984.0),
        "1.8014398509481984e+16"
    );
}
