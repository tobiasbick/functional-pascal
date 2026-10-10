//! `case Value of when Labels: Expression; ... [else Expression;] end case` in expression positions.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`

use super::*;
use crate::ParseDiagnostic;
use crate::tests::{parse_ok, parse_with_errors};
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_STATEMENT_IN_EXPRESSION_BRANCH};

fn parser_errors(source: &str) -> Vec<crate::ParseError> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .cloned()
        .collect()
}

#[test]
fn arms_guards_patterns_and_else_keep_their_shape() {
    let Expr::Case { arms, else_arm, .. } = parse_expr(
        "case Value of when 1, 2: 'low'; when 3..9 if Ready: 'mid'; when Some(const N): N; else 'other'; end case",
    ) else {
        panic!("expected a case expression");
    };
    assert_eq!(arms.len(), 3);
    assert_eq!(arms[0].labels.len(), 2);
    assert!(arms[1].guard.is_some());
    assert!(matches!(
        arms[1].labels[0],
        CaseLabel::Value { end: Some(_), .. }
    ));
    assert!(matches!(arms[2].labels[0], CaseLabel::Pattern(_)));
    assert!(matches!(
        else_arm.as_deref(),
        Some(CaseExprElse {
            value: Expr::Str(..),
            ..
        })
    ));
}

#[test]
fn case_expressions_are_operands_and_return_values() {
    let Expr::BinaryOp {
        op: BinaryOp::Add,
        right,
        ..
    } = parse_expr("1 + case X of when 1: 2; else 3; end case")
    else {
        panic!("expected `+` at the top");
    };
    assert!(matches!(*right, Expr::Case { .. }));
    let program = parse_ok(
        "program T; function F(): integer; begin return case X of when 1: 2; else 3; end case; end function; begin case X of when 1: null; else null; end case; end.",
    );
    assert!(matches!(program.body.first(), Some(Stmt::Case { .. })));
}

#[test]
fn statements_missing_terminators_and_missing_endings_are_diagnosed() {
    let errors = parser_errors(
        "program T; begin const X: integer := case 1 of when 1: Y := 1; else 2; end case; end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, PARSE_STATEMENT_IN_EXPRESSION_BRANCH);
    let errors = parser_errors(
        "program T; begin const X: integer := case 1 of when 1: 1 else 2; end case; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == PARSE_EXPECTED_TOKEN && error.message.contains('`')),
        "{errors:#?}"
    );
    let errors = parser_errors("program T; begin const X: integer := case 1 of when 1: 1; end.");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("end case")),
        "{errors:#?}"
    );
}
