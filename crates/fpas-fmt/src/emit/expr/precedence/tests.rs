//! Arithmetic chains keep their left grouping without redundant parentheses.

use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr, Stmt, parse};

use super::super::format_expr;

fn formatted_initializer(expression: &str) -> String {
    let (program, errors) = parse(&format!(
        "program T; begin const X: integer := {expression}; end."
    ));
    assert!(errors.is_empty(), "{errors:?}");
    let Stmt::Const(binding) = &program.body[0] else {
        panic!("expected a const statement");
    };
    format_expr(&binding.value)
}

const SPAN: Span = Span {
    offset: 0,
    length: 0,
    line: 1,
    column: 1,
    source_id: 0,
};

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
        span: SPAN,
    }
}

fn integer(value: i64) -> Expr {
    Expr::Integer(value, SPAN)
}

#[test]
fn left_grouped_chains_need_no_parentheses() {
    for source in [
        "2 * 3 * 4",
        "10 - 5 - 2",
        "1 + 2 - 3",
        "8 div 2 * 3",
        "A / B * C",
    ] {
        assert_eq!(formatted_initializer(source), source);
    }
}

#[test]
fn written_parentheses_are_kept() {
    for source in ["10 - (5 - 2)", "(10 - 5) - 2", "100 div (10 div 2)"] {
        assert_eq!(formatted_initializer(source), source);
    }
}

#[test]
fn right_nested_operands_without_parentheses_are_grouped() {
    let right_nested = binary(
        BinaryOp::Sub,
        integer(10),
        binary(BinaryOp::Sub, integer(5), integer(2)),
    );
    assert_eq!(format_expr(&right_nested), "10 - (5 - 2)");
    let left_nested = binary(
        BinaryOp::Sub,
        binary(BinaryOp::Sub, integer(10), integer(5)),
        integer(2),
    );
    assert_eq!(format_expr(&left_nested), "10 - 5 - 2");
}
