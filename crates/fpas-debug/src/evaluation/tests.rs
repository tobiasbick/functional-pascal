use fpas_vm::DebugEvaluationLimits;

use super::{LogMessage, LogMessageLimits, LogSegment, parse_debug_expression};

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
    ];
    for source in expressions {
        assert!(
            parse_debug_expression(source, DebugEvaluationLimits::default()).is_ok(),
            "expected supported expression: {source}"
        );
    }
}

#[test]
fn validator_rejects_every_effectful_or_constructing_category() {
    let expressions = [
        "Call()",
        "Value.Method()",
        "[]",
        "[: ]",
        "record X := 1; end",
        "Ok(1)",
        "Error('x')",
        "Some(1)",
        "None",
        "nil",
        "go Work()",
        "try Work()",
        "Point with X := 2; end",
        "function(): integer begin return 1 end",
    ];
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

    for source in ["{}", "{Counter", "Counter}", "{Counter{Other}}", "{Call()}"] {
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
