//! Formatting preserves logical grouping, including ASTs without explicit parentheses.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

use fpas_parser::{Expr, Stmt, parse};

fn expression(program: &fpas_parser::Program) -> &Expr {
    let Stmt::Return(Some(expr), _) = &program.body[0] else {
        panic!("return fixture")
    };
    expr
}

fn strip_parentheses(expr: &mut Expr) {
    while let Expr::Paren(inner, _) = expr {
        *expr = *inner.clone();
    }
    match expr {
        Expr::BinaryOp { left, right, .. } => {
            strip_parentheses(left);
            strip_parentheses(right);
        }
        Expr::UnaryOp { operand, .. } | Expr::Try(operand, _) => strip_parentheses(operand),
        _ => {}
    }
}

fn shape(expr: &Expr) -> String {
    match expr {
        Expr::Paren(inner, _) => shape(inner),
        Expr::BinaryOp {
            op, left, right, ..
        } => format!("{op:?}({}, {})", shape(left), shape(right)),
        Expr::UnaryOp { op, operand, .. } => format!("{op:?}({})", shape(operand)),
        Expr::Try(inner, _) => format!("Try({})", shape(inner)),
        Expr::Designator(d) => match &d.parts[0] {
            fpas_parser::DesignatorPart::Ident(name, _) => name.clone(),
            _ => panic!("named operand"),
        },
        _ => panic!("unexpected fixture expression: {expr:?}"),
    }
}

fn round_trip_ast(source: &str) -> String {
    let (mut program, errors) = parse(&format!("program T; begin return {source}; end."));
    assert!(errors.is_empty(), "{source}: {errors:?}");
    let Stmt::Return(Some(expr), _) = &mut program.body[0] else {
        panic!("fixture")
    };
    strip_parentheses(expr);
    let expected = shape(expression(&program));
    let formatted = fpas_fmt::format_program(&program);
    let (again, errors) = parse(&formatted);
    assert!(errors.is_empty(), "{formatted}: {errors:?}");
    assert_eq!(shape(expression(&again)), expected, "{formatted}");
    assert_eq!(
        fpas_fmt::format_program(&again),
        formatted,
        "idempotent: {source}"
    );
    formatted
}

#[test]
fn every_logical_pair_gets_the_parentheses_required_by_the_parser() {
    for first in ["and", "or", "xor"] {
        for second in ["and", "or", "xor"] {
            for source in [
                format!("(A {first} B) {second} C"),
                format!("A {first} (B {second} C)"),
            ] {
                let formatted = round_trip_ast(&source);
                if first != second {
                    assert!(formatted.contains(&source), "{formatted}");
                }
            }
        }
    }
}

#[test]
fn negation_comparisons_and_try_preserve_their_ast_grouping() {
    for source in [
        "not A = B",
        "(not A) = B",
        "A = (not B)",
        "not A and B",
        "not (A and B)",
        "not not A = B",
        "try (not A)",
        "-(not A)",
        "-(A + B)",
        "not try A = B",
        "(A < B) = C",
        "A = (B < C)",
        "A < B and B < C",
        "A + B * C",
    ] {
        round_trip_ast(source);
    }
}

#[test]
fn wrapped_logical_expressions_keep_mixed_grouping_and_comments() {
    let source = "program T; begin\n// keep this condition\nreturn (VeryLongIdentifierAlpha = VeryLongIdentifierBeta and VeryLongIdentifierGamma < VeryLongIdentifierDelta) or not VeryLongIdentifierEpsilon = VeryLongIdentifierZeta; end.";
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("matching source");
    assert_eq!(formatted.matches("// keep this condition").count(), 1);
    assert!(formatted.contains(" or\n"), "{formatted}");
    let (again, errors) = fpas_parser::parse_compilation_unit(&formatted);
    assert!(errors.is_empty(), "{formatted}: {errors:?}");
    assert_eq!(
        fpas_fmt::format_source(&formatted, &again).expect("formatted source"),
        formatted
    );
    let (before, _) = parse(source);
    let (after, _) = parse(&formatted);
    assert_eq!(shape(expression(&before)), shape(expression(&after)));
}
