#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "parser tests use direct assertions to keep failures local"
)]

use fpas_vm::{DebugAssignmentSelector, DebugEvaluationLimits, DebugExpression};

use super::{
    LogMessage, LogMessageLimits, LogSegment, parse_debug_assignment_target, parse_debug_expression,
};

#[test]
fn assignment_target_parser_preserves_named_stored_selector_order() {
    let target = parse_debug_assignment_target(
        "State.Items[Selected + 1].Value",
        DebugEvaluationLimits::default(),
    )
    .expect("valid assignment target");
    assert_eq!(target.root, "State");
    assert!(matches!(
        target.selectors.as_slice(),
        [
            DebugAssignmentSelector::Field(items),
            DebugAssignmentSelector::Index(_),
            DebugAssignmentSelector::Field(value)
        ] if items == "Items" && value == "Value"
    ));

    let qualified =
        parse_debug_assignment_target("Optional.Some.value", DebugEvaluationLimits::default())
            .expect("qualified option target");
    assert_eq!(qualified.root, "Optional");
    assert!(matches!(
        qualified.selectors.as_slice(),
        [
            DebugAssignmentSelector::Field(variant),
            DebugAssignmentSelector::Field(payload)
        ] if variant == "Some" && payload == "value"
    ));

    let nested =
        parse_debug_assignment_target("Holder.Item.Count.Value", DebugEvaluationLimits::default())
            .expect("nested qualified enum target");
    assert_eq!(nested.root, "Holder");
    assert!(matches!(
        nested.selectors.as_slice(),
        [
            DebugAssignmentSelector::Field(item),
            DebugAssignmentSelector::Field(variant),
            DebugAssignmentSelector::Field(payload)
        ] if item == "Item" && variant == "Count" && payload == "Value"
    ));

    let indexed =
        parse_debug_assignment_target("Items[0].Some.value", DebugEvaluationLimits::default())
            .expect("indexed qualified option target");
    assert_eq!(indexed.root, "Items");
    assert!(matches!(
        indexed.selectors.as_slice(),
        [
            DebugAssignmentSelector::Index(_),
            DebugAssignmentSelector::Field(variant),
            DebugAssignmentSelector::Field(payload)
        ] if variant == "Some" && payload == "value"
    ));

    let mixed_variant =
        parse_debug_assignment_target("sElEcTeD.cOuNt.vAlUe", DebugEvaluationLimits::default())
            .expect("mixed-case qualified variant target");
    assert_eq!(mixed_variant.root, "sElEcTeD");
    assert!(matches!(
        mixed_variant.selectors.as_slice(),
        [
            DebugAssignmentSelector::Field(variant),
            DebugAssignmentSelector::Field(payload)
        ] if variant == "cOuNt" && payload == "vAlUe"
    ));

    for source in [
        "Counter",
        "Origin.X",
        "Items[Index + 1]",
        "Scores['blue']",
        "sTaTe.iTeMs[Selected].vAlUe",
        "Outcome.Ok.value",
        "Outcome.Error.value",
    ] {
        parse_debug_assignment_target(source, DebugEvaluationLimits::default())
            .unwrap_or_else(|error| panic!("valid target {source}: {error:?}"));
    }

    let nested_reserved = parse_debug_assignment_target(
        "Items[Optional.Some.value]",
        DebugEvaluationLimits::default(),
    )
    .expect_err("reserved words inside index expressions remain reserved");
    assert_eq!(nested_reserved.code, "expression_target_parse");

    let mixed_case = parse_debug_assignment_target(
        "sTaTe.iTeMs[Selected].vAlUe",
        DebugEvaluationLimits::default(),
    )
    .expect("mixed-case target");
    assert_eq!(mixed_case.root, "sTaTe");
    assert!(matches!(
        mixed_case.selectors.as_slice(),
        [
            DebugAssignmentSelector::Field(items),
            DebugAssignmentSelector::Index(_),
            DebugAssignmentSelector::Field(value)
        ] if items == "iTeMs" && value == "vAlUe"
    ));
}

#[test]
fn assignment_target_parser_rejects_computed_and_malformed_targets() {
    for source in [
        "(Items)[0]",
        "Build()[0]",
        "Value.Method()",
        "1 + Counter",
        "'text'",
    ] {
        let error = parse_debug_assignment_target(source, DebugEvaluationLimits::default())
            .expect_err("unsupported assignment target");
        assert_eq!(error.code, "expression_target_unsupported", "{source}");
    }

    for source in ["", "Items[", "Counter := 1"] {
        let error = parse_debug_assignment_target(source, DebugEvaluationLimits::default())
            .expect_err("malformed assignment target");
        assert_eq!(error.code, "expression_target_parse", "{source}");
        assert!(!error.hint.is_empty(), "{source}");
    }

    let unicode = "Ätems['unterminated]";
    let error = parse_debug_assignment_target(unicode, DebugEvaluationLimits::default())
        .expect_err("unterminated UTF-8 target");
    assert_eq!(error.code, "expression_target_parse");
    assert!(error.offset <= unicode.len());
    assert!(error.length <= unicode.len().saturating_sub(error.offset));
}

#[test]
fn assignment_target_parser_reuses_expression_limits() {
    let byte_limited = DebugEvaluationLimits {
        max_expression_bytes: 4,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        parse_debug_assignment_target("Items[0]", byte_limited)
            .expect_err("target byte limit")
            .code,
        "evaluation_limit"
    );

    let depth_limited = DebugEvaluationLimits {
        max_depth: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        parse_debug_assignment_target("Items[Other[0]]", depth_limited)
            .expect_err("target depth limit")
            .code,
        "evaluation_limit"
    );

    let operation_limited = DebugEvaluationLimits {
        max_operations: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        parse_debug_assignment_target("Items[Index + 1]", operation_limited)
            .expect_err("target operation limit")
            .code,
        "evaluation_limit"
    );

    let traversal_limited = DebugEvaluationLimits {
        max_traversals: 0,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        parse_debug_assignment_target("Items[Other[0]]", traversal_limited)
            .expect_err("target traversal limit")
            .code,
        "evaluation_limit"
    );
}

#[test]
fn validator_accepts_the_complete_read_only_category_matrix() {
    let expressions = [
        "1",
        "1.5",
        "true",
        "'text'",
        "VisibleName",
        "(VisibleName)",
        "-1",
        "not false",
        "1 + 2",
        "3 - 2",
        "2 * 3",
        "4 / 2",
        "4 div 2",
        "5 mod 2",
        "true and false",
        "true or false",
        "1 xor 2",
        "1 shl 2",
        "4 shr 1",
        "1 = 1",
        "1 <> 2",
        "1 < 2",
        "1 <= 2",
        "2 > 1",
        "2 >= 1",
        "1 in Values",
        "Point.X",
        "Items[0]",
        "Dictionary['key']",
        "Text[0]",
        "Call()",
        "Value.Method()",
        "[]",
        "[: ]",
        "record X := 1; end",
        "Ok(1)",
        "Error('x')",
        "Some(1)",
        "None",
        "Choice.Empty",
        "Choice.Pair(1, 2)",
        "cHoIcE.pAiR(1, 2)",
        "try Work()",
        "Point with X := 2; end",
    ];
    for source in expressions {
        assert!(
            parse_debug_expression(source, DebugEvaluationLimits::default()).is_ok(),
            "expected supported expression: {source}"
        );
    }
}

#[test]
fn validator_lowers_qualified_enum_constructors_to_call_and_field_forms() {
    let empty = parse_debug_expression("Choice.Empty", DebugEvaluationLimits::default())
        .expect("fieldless constructor");
    assert!(
        matches!(
            empty,
            DebugExpression::Field { ref name, .. } if name == "Empty"
        ),
        "{empty:?}"
    );

    let pair = parse_debug_expression("Choice.Pair(1, 2)", DebugEvaluationLimits::default())
        .expect("data-carrying constructor");
    assert!(
        matches!(
            pair,
            DebugExpression::Call { ref callee, ref arguments }
                if matches!(callee.as_ref(), DebugExpression::Callable(name) if name == "Choice.Pair")
                    && arguments.len() == 2
        ),
        "{pair:?}"
    );
}

#[test]
fn validator_rejects_every_effectful_or_constructing_category() {
    let expressions = ["nil", "go Work()", "function(): integer begin return 1 end"];
    for source in expressions {
        let error = parse_debug_expression(source, DebugEvaluationLimits::default())
            .expect_err("unsupported expression category");
        assert!(!error.code.is_empty(), "stable code for {source}");
        assert!(!error.hint.is_empty(), "actionable hint for {source}");
    }
}

#[test]
fn expression_source_limit_uses_utf8_bytes_without_truncation() {
    let limits = DebugEvaluationLimits::default();
    let exact = format!("'{}'", "a".repeat(limits.max_expression_bytes - 2));
    assert_eq!(exact.len(), limits.max_expression_bytes);
    assert!(parse_debug_expression(&exact, limits).is_ok());

    let excessive = format!("{exact}a");
    let error = parse_debug_expression(&excessive, limits).expect_err("source byte limit");
    assert_eq!(error.code, "evaluation_limit");
}

#[test]
fn log_templates_escape_braces_and_preparse_bounded_expressions() {
    let message = LogMessage::parse(
        "value={Counter + 1} {{ready}}",
        LogMessageLimits::default(),
        DebugEvaluationLimits::default(),
    )
    .expect("valid log template");
    assert_eq!(message.segments().len(), 3);
    assert!(matches!(message.segments()[1], LogSegment::Expression(_)));

    for source in ["{}", "{Counter", "Counter}", "{Counter{Other}}"] {
        assert!(
            LogMessage::parse(
                source,
                LogMessageLimits::default(),
                DebugEvaluationLimits::default(),
            )
            .is_err(),
            "invalid template: {source}"
        );
    }
}
