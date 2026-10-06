//! Logical precedence boundaries, mixing errors, and comparison recovery.

use crate::{Expr, ParseDiagnostic, Stmt, parse, parse_expression};
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;

fn shape(expr: &Expr) -> String {
    match expr {
        Expr::BinaryOp {
            op, left, right, ..
        } => {
            format!("{op:?}({}, {})", shape(left), shape(right))
        }
        Expr::UnaryOp { op, operand, .. } => format!("{op:?}({})", shape(operand)),
        Expr::Try(operand, _) => format!("Try({})", shape(operand)),
        Expr::Paren(operand, _) => format!("Paren({})", shape(operand)),
        Expr::Designator(d) => match &d.parts[0] {
            crate::DesignatorPart::Ident(name, _) => name.clone(),
            _ => panic!("expected named operand"),
        },
        _ => panic!("unexpected expression: {expr:?}"),
    }
}

#[test]
fn every_precedence_boundary_and_not_placement_has_the_agreed_grouping() {
    for (source, expected) in [
        ("A + B * C", "Add(A, Mul(B, C))"),
        ("-A * B", "Mul(Negate(A), B)"),
        ("try A + B", "Add(Try(A), B)"),
        ("A + B = C", "Eq(Add(A, B), C)"),
        ("not A = B", "Not(Eq(A, B))"),
        ("not A + B > C", "Not(Gt(Add(A, B), C))"),
        ("not A and B", "And(Not(A), B)"),
        ("A and not B = C", "And(A, Not(Eq(B, C)))"),
        ("not not A = B", "Not(Not(Eq(A, B)))"),
        ("(not A) = B", "Eq(Paren(Not(A)), B)"),
        ("A = (not B)", "Eq(A, Paren(Not(B)))"),
        ("not try A = B", "Not(Eq(Try(A), B))"),
        ("try (not A)", "Try(Paren(Not(A)))"),
        ("-(not A)", "Negate(Paren(Not(A)))"),
    ] {
        let (expr, errors) = parse_expression(source);
        assert!(errors.is_empty(), "{source}: {errors:?}");
        assert_eq!(shape(&expr), expected, "{source}");
    }
    for source in ["A = not B", "try not A", "-not A"] {
        assert!(!parse_expression(source).1.is_empty(), "{source}");
    }
}

#[test]
fn postfix_calls_indexes_fields_and_record_updates_bind_above_prefix_operators() {
    let (expr, errors) = parse_expression("-Make().Value");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        matches!(expr, Expr::UnaryOp { operand, .. } if matches!(*operand, Expr::Postfix { .. }))
    );
    for source in [
        "not Values[0] = Expected",
        "not Make().Flag = Expected",
        "not Base with Flag := true; end with = Expected",
    ] {
        let (expr, errors) = parse_expression(source);
        assert!(errors.is_empty(), "{source}: {errors:?}");
        let Expr::UnaryOp { operand, .. } = expr else {
            panic!("{source}")
        };
        assert!(matches!(*operand, Expr::BinaryOp { .. }), "{source}");
    }
}

#[test]
fn every_comparison_binds_above_not_and_each_logical_operator() {
    for comparison in ["=", "<>", "<", ">", "<=", ">=", "in"] {
        for logical in ["and", "or", "xor"] {
            let source = format!("not A {comparison} B {logical} C {comparison} D");
            let (expr, errors) = parse_expression(&source);
            assert!(errors.is_empty(), "{source}: {errors:?}");
            let Expr::BinaryOp { left, right, .. } = expr else {
                panic!("{source}")
            };
            assert!(
                matches!(*left, Expr::UnaryOp { operand, .. } if matches!(*operand, Expr::BinaryOp { .. }))
            );
            assert!(matches!(*right, Expr::BinaryOp { .. }));
        }
    }
}

#[test]
fn all_mixed_logical_pairs_require_parentheses_and_recover_to_the_next_statement() {
    for first in ["and", "or", "xor"] {
        for second in ["and", "or", "xor"] {
            let source = format!("A {first} B {second} C");
            let (expr, errors) = parse_expression(&source);
            if first == second {
                assert!(errors.is_empty(), "{source}: {errors:?}");
                assert!(
                    matches!(expr, Expr::BinaryOp { left, .. } if matches!(*left, Expr::BinaryOp { .. }))
                );
                continue;
            }
            assert_eq!(errors.len(), 1, "{source}: {errors:?}");
            let diagnostic = errors[0].as_diagnostic();
            assert_eq!(diagnostic.code, PARSE_EXPECTED_EXPRESSION);
            assert!(diagnostic.message.contains("require parentheses"));
            assert!(
                diagnostic
                    .help
                    .as_ref()
                    .unwrap()
                    .contains(&format!("(A {first} B) {second} C"))
            );
            let span = diagnostic.span.unwrap();
            assert_eq!(span.offset(), source.rfind(second).unwrap());
            assert_eq!(span.length(), second.len());
            for grouped in [
                format!("(A {first} B) {second} C"),
                format!("A {first} (B {second} C)"),
            ] {
                assert!(parse_expression(&grouped).1.is_empty(), "{grouped}");
            }
            let (program, errors) =
                parse(&format!("program T; begin X := {source}; Y := true; end."));
            assert_eq!(
                errors
                    .iter()
                    .filter_map(ParseDiagnostic::as_parser_error)
                    .count(),
                1
            );
            assert_eq!(program.body.len(), 2);
            assert!(matches!(program.body[1], Stmt::Assign { .. }));
        }
    }
}

#[test]
fn all_comparison_pairs_are_non_associative_and_suggest_separate_comparisons() {
    for first in ["=", "<>", "<", ">", "<=", ">=", "in"] {
        for second in ["=", "<>", "<", ">", "<=", ">=", "in"] {
            let source = format!("A {first} B {second} C");
            let (_, errors) = parse_expression(&source);
            assert_eq!(errors.len(), 1, "{source}: {errors:?}");
            assert!(
                errors[0]
                    .as_diagnostic()
                    .help
                    .as_ref()
                    .unwrap()
                    .contains("A < B and B < C")
            );
        }
    }
    for source in ["(A < B) = C", "A = (B < C)", "A < B and B < C"] {
        assert!(parse_expression(source).1.is_empty(), "{source}");
    }
}
