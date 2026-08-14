use super::*;
use fpas_bytecode::{EnumTypeId, RecordTypeId};

fn types(entries: Vec<DebugType>) -> Vec<DebugType> {
    entries
}

#[test]
fn matching_function_types_are_compatible() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
    ]);
    require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(1)).expect("same node");
}

#[test]
fn parameter_count_mismatch_is_rejected() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: Vec::new(),
            result: DebugTypeId::new(0),
        },
    ]);
    let error =
        require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(2)).expect_err("arity");
    assert_eq!(error.kind, DebugErrorKind::VariableValueType);
    assert!(error.message.contains("parameter count"), "{error:?}");
    assert!(error.hint.contains("signature"), "{}", error.hint);
}

#[test]
fn parameter_and_result_mismatches_are_rejected() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Boolean,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(1)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(1),
        },
    ]);
    let parameter =
        require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(3)).expect_err("param");
    assert!(parameter.message.contains("parameter 1"), "{parameter:?}");
    let result =
        require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(4)).expect_err("result");
    assert!(result.message.contains("result type"), "{result:?}");
}

#[test]
fn nested_function_and_layout_identity_are_compared_structurally() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(1)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(5)],
            result: DebugTypeId::new(0),
        },
        DebugType::Record(RecordTypeId::new(0)),
        DebugType::Function {
            parameters: vec![DebugTypeId::new(4)],
            result: DebugTypeId::new(0),
        },
        DebugType::Record(RecordTypeId::new(1)),
        DebugType::Enum(EnumTypeId::new(0)),
        DebugType::Function {
            parameters: vec![DebugTypeId::new(7)],
            result: DebugTypeId::new(0),
        },
        DebugType::Enum(EnumTypeId::new(1)),
        DebugType::Function {
            parameters: vec![DebugTypeId::new(9)],
            result: DebugTypeId::new(0),
        },
    ]);
    require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(2)).expect("nested same");
    assert!(
        require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(3))
            .expect_err("nested")
            .message
            .contains("parameter 1")
    );
    assert!(
        require_compatible(&types, DebugTypeId::new(5), DebugTypeId::new(3))
            .expect_err("record")
            .hint
            .contains("signature")
    );
    assert!(
        require_compatible(&types, DebugTypeId::new(8), DebugTypeId::new(10))
            .expect_err("enum")
            .kind
            == DebugErrorKind::VariableValueType
    );
}

#[test]
fn recursive_function_graphs_terminate() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(2),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(1),
        },
    ]);
    require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(2)).expect("recursive");
}

#[test]
fn malformed_type_ids_are_rejected() {
    let types = types(vec![DebugType::Integer]);
    let error =
        require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(1)).expect_err("missing");
    assert_eq!(error.kind, DebugErrorKind::VariableValueType);
    assert!(error.message.contains("unavailable"), "{error:?}");
}

#[test]
fn signature_traversal_obeys_depth_limits() {
    let types = types(vec![
        DebugType::Integer,
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(0)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(1)],
            result: DebugTypeId::new(0),
        },
        DebugType::Function {
            parameters: vec![DebugTypeId::new(2)],
            result: DebugTypeId::new(0),
        },
    ]);
    require_compatible_bounded(&types, DebugTypeId::new(3), DebugTypeId::new(4), 8, 64)
        .expect("within limit");
    let error = require_compatible_bounded(&types, DebugTypeId::new(3), DebugTypeId::new(4), 0, 64)
        .expect_err("depth");
    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
}
