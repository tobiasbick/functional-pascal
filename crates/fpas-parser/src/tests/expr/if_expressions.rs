//! `if C then A elsif D then B else E end if` in expression positions.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`

use super::*;
use crate::ParseDiagnostic;
use crate::tests::{parse_ok, parse_with_errors};
use fpas_diagnostics::codes::{
    PARSE_EXPECTED_TOKEN, PARSE_IF_EXPRESSION_WITHOUT_ELSE, PARSE_STATEMENT_IN_EXPRESSION_BRANCH,
};

fn parser_errors(source: &str) -> Vec<crate::ParseError> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .cloned()
        .collect()
}

#[test]
fn elsif_chains_keep_branches_in_source_order() {
    let Expr::If {
        branches,
        else_value,
        ..
    } = parse_expr("if A then 1 elsif B then 2 elsif C then 3 else 4 end if")
    else {
        panic!("expected an if expression");
    };
    let values = branches
        .iter()
        .map(|branch| match branch.value {
            Expr::Integer(value, _) => value,
            _ => panic!("{branch:#?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [1, 2, 3]);
    assert!(matches!(*else_value, Expr::Integer(4, _)));
}

#[test]
fn if_expressions_are_primary_operands_and_nest() {
    let Expr::BinaryOp {
        op: BinaryOp::Add,
        right,
        ..
    } = parse_expr("1 + if C then 2 else 3 end if * 4")
    else {
        panic!("expected `+` at the top");
    };
    assert!(matches!(
        *right,
        Expr::BinaryOp {
            op: BinaryOp::Mul,
            ..
        }
    ));
    let Expr::If { branches, .. } = parse_expr("if A then if B then 1 else 2 end if else 3 end if")
    else {
        panic!("expected an if expression");
    };
    assert!(matches!(branches[0].value, Expr::If { .. }));
}

#[test]
fn conditions_accept_is_tests() {
    let Expr::If { branches, .. } =
        parse_expr("if X is Some(const V) and V > 0 then V else 0 end if")
    else {
        panic!("expected an if expression");
    };
    assert!(matches!(
        branches[0].condition,
        Expr::BinaryOp {
            op: BinaryOp::And,
            ..
        }
    ));
}

#[test]
fn statement_start_stays_the_if_statement_and_return_takes_the_expression() {
    let program = parse_ok(
        "program T; function F(): integer; begin return if A then 1 else 2 end if; end function; begin if A then null; end if; end.",
    );
    assert!(matches!(program.body.first(), Some(Stmt::If { .. })));
}

#[test]
fn missing_else_and_statements_in_branches_are_diagnosed() {
    let errors = parser_errors("program T; begin const X: integer := if A then 1 end if; end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, PARSE_IF_EXPRESSION_WITHOUT_ELSE);
    for source in [
        "program T; begin const X: integer := if A then 1; else 2 end if; end.",
        "program T; begin const X: integer := if A then Y := 1 else 2 end if; end.",
    ] {
        let errors = parser_errors(source);
        assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
        assert_eq!(
            errors[0].code, PARSE_STATEMENT_IN_EXPRESSION_BRANCH,
            "{source}"
        );
    }
    let errors = parser_errors("program T; begin const X: integer := if A then 1 else 2; end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == PARSE_EXPECTED_TOKEN && error.message.contains("end if")),
        "{errors:#?}"
    );
}
