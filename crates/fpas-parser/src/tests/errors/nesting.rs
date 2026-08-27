use crate::parser::MAX_PARSER_NESTING_DEPTH;
use crate::{ParseDiagnostic, parse, parse_expression};
use fpas_diagnostics::codes::PARSE_NESTING_LIMIT_EXCEEDED;

fn has_nesting_limit(diagnostics: &[ParseDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().code == PARSE_NESTING_LIMIT_EXCEEDED)
}

fn assert_only_nesting_limit(diagnostics: &[ParseDiagnostic]) {
    assert_eq!(diagnostics.len(), 1, "unexpected: {diagnostics:#?}");
    assert!(has_nesting_limit(diagnostics));
}

#[test]
fn excessive_prefix_nesting_reports_a_controlled_diagnostic() {
    let source = format!("{}true", "not ".repeat(MAX_PARSER_NESTING_DEPTH - 1));

    let (_, diagnostics) = parse_expression(&source);

    assert_only_nesting_limit(&diagnostics);
}

#[test]
fn expression_just_below_the_nesting_limit_remains_accepted() {
    let source = format!("{}true", "not ".repeat(MAX_PARSER_NESTING_DEPTH - 2));

    let (_, diagnostics) = parse_expression(&source);

    assert!(diagnostics.is_empty(), "unexpected: {diagnostics:#?}");
}

#[test]
fn mixed_routine_and_statement_nesting_shares_one_budget() {
    let routine_depth = MAX_PARSER_NESTING_DEPTH / 2;
    let statement_depth = MAX_PARSER_NESTING_DEPTH / 2 + 1;
    let mut source = String::from("program T; ");
    for index in 0..routine_depth {
        source.push_str(&format!("procedure P{index}(); "));
    }
    source.push_str("begin ");
    source.push_str(&"if true then ".repeat(statement_depth));
    source.push_str("Value := 1 end; ");
    source.push_str(&"begin end; ".repeat(routine_depth - 1));
    source.push_str("begin end.");

    let (_, diagnostics) = parse(&source);

    assert_only_nesting_limit(&diagnostics);
}

#[test]
fn excessive_statement_nesting_reports_the_shared_limit() {
    let source = format!(
        "program T; begin {}Value := 1 end.",
        "if true then ".repeat(MAX_PARSER_NESTING_DEPTH + 1)
    );

    let (_, diagnostics) = parse(&source);

    assert_only_nesting_limit(&diagnostics);
}

#[test]
fn excessive_type_nesting_reports_the_shared_limit() {
    let source = format!(
        "program T; type Deep = {}integer; begin end.",
        "array of ".repeat(MAX_PARSER_NESTING_DEPTH + 1)
    );

    let (_, diagnostics) = parse(&source);

    assert_only_nesting_limit(&diagnostics);
}

#[test]
fn excessive_routine_nesting_reports_the_shared_limit() {
    let mut source = String::from("program T; ");
    for index in 0..=MAX_PARSER_NESTING_DEPTH {
        source.push_str(&format!("procedure P{index}(); "));
    }
    source.push_str(&"begin end; ".repeat(MAX_PARSER_NESTING_DEPTH + 1));
    source.push_str("begin end.");

    let (_, diagnostics) = parse(&source);

    assert_only_nesting_limit(&diagnostics);
}
