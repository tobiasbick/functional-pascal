//! Operator precedence, migration diagnostics, and structural regression coverage.
use fpas_parser::{BinaryOp as B, Expr, Stmt, UnaryOp, parse, parse_expression};

fn expression(source: &str) -> Expr {
    let (expr, errors) = parse_expression(source);
    assert!(errors.is_empty(), "{source}: {errors:?}");
    expr
}

#[test]
fn comparisons_bind_above_not_and_logical_operators() {
    let Expr::BinaryOp {
        op: B::And,
        left,
        right,
        ..
    } = expression("not X > 0 and Y + 2 * 3 > 0")
    else {
        panic!("and root");
    };
    assert!(
        matches!(*left, Expr::UnaryOp { op: UnaryOp::Not, operand, .. } if matches!(*operand, Expr::BinaryOp { op: B::Gt, .. }))
    );
    assert!(matches!(*right, Expr::BinaryOp { op: B::Gt, .. }));
    assert!(
        matches!(expression("not not A in Items"), Expr::UnaryOp { operand, .. } if matches!(&*operand, Expr::UnaryOp { operand, .. } if matches!(**operand, Expr::BinaryOp { op: B::In, .. })))
    );
}

#[test]
fn same_operator_chains_are_left_associative() {
    for (op, expected) in [("and", B::And), ("or", B::Or), ("xor", B::Xor)] {
        let Expr::BinaryOp {
            op: actual, left, ..
        } = expression(&format!("A {op} B {op} C"))
        else {
            panic!("binary");
        };
        assert_eq!(actual, expected);
        assert!(matches!(*left, Expr::BinaryOp { op: child, .. } if child == expected));
    }
}

#[test]
fn every_arithmetic_and_comparison_operator_respects_its_boundary() {
    for (multiplicative, expected) in [
        ("*", B::Mul),
        ("/", B::RealDiv),
        ("div", B::IntDiv),
        ("mod", B::Mod),
    ] {
        for (additive, additive_op) in [("+", B::Add), ("-", B::Sub)] {
            let Expr::BinaryOp { op, right, .. } =
                expression(&format!("A {additive} B {multiplicative} C"))
            else {
                panic!("additive root");
            };
            assert_eq!(op, additive_op);
            assert!(matches!(*right, Expr::BinaryOp { op, .. } if op == expected));
        }
        let Expr::BinaryOp { op, left, .. } =
            expression(&format!("A {multiplicative} B {multiplicative} C"))
        else {
            panic!("multiplicative root");
        };
        assert_eq!(op, expected);
        assert!(matches!(*left, Expr::BinaryOp { op, .. } if op == expected));
    }
    for (comparison, expected) in [
        ("=", B::Eq),
        ("<>", B::NotEq),
        ("<", B::Lt),
        (">", B::Gt),
        ("<=", B::LtEq),
        (">=", B::GtEq),
        ("in", B::In),
    ] {
        let Expr::UnaryOp {
            op: UnaryOp::Not,
            operand,
            ..
        } = expression(&format!("not A + B {comparison} C"))
        else {
            panic!("not root");
        };
        assert!(
            matches!(*operand, Expr::BinaryOp { op, left, .. } if op == expected && matches!(*left, Expr::BinaryOp { op: B::Add, .. }))
        );
        for second in ["=", "<>", "<", ">", "<=", ">=", "in"] {
            let source = format!("A {comparison} B {second} C");
            let (_, errors) = parse_expression(&source);
            assert!(
                errors.iter().any(|error| error
                    .as_parser_error()
                    .is_some_and(|error| error.message.contains("Chained comparison"))),
                "{source}: {errors:#?}"
            );
        }
    }
}

#[test]
fn postfix_chains_and_record_updates_bind_above_unary_and_arithmetic() {
    for source in ["-Get()[0].Value * 2", "try Get()[0].Value * 2"] {
        let Expr::BinaryOp {
            op: B::Mul, left, ..
        } = expression(source)
        else {
            panic!("multiply root");
        };
        let operand = match *left {
            Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand,
                ..
            }
            | Expr::Try(operand, _) => operand,
            other => panic!("prefix operand: {other:#?}"),
        };
        assert!(matches!(*operand, Expr::Postfix { operations, .. } if operations.len() == 2));
    }
    let Expr::BinaryOp {
        op: B::Add, left, ..
    } = expression("-Value with X := 1 + 2; end with + 3")
    else {
        panic!("add root");
    };
    assert!(
        matches!(*left, Expr::UnaryOp { operand, .. } if matches!(*operand, Expr::RecordUpdate { .. }))
    );
}

#[test]
fn every_mixed_pair_requires_explicit_grouping() {
    for first in ["and", "or", "xor"] {
        for second in ["and", "or", "xor"] {
            if first == second {
                continue;
            }
            let (_, errors) = parse_expression(&format!("A {first} B {second} C"));
            assert!(errors.iter().any(|e| {
                e.as_parser_error()
                    .is_some_and(|e| e.message.contains("Mixed logical operators"))
            }));
            expression(&format!("(A {first} B) {second} C"));
            expression(&format!("A {first} (B {second} C)"));
        }
    }
}

#[test]
fn migration_diagnostics_recover_without_duplicating_effects() {
    for bad in [
        "A < Next() < C",
        "A in B in C",
        "A and B or C",
        "1 ShL 2",
        "4 sHr 1",
    ] {
        let (program, errors) = parse(&format!(
            "program T; begin X := {bad}; Y := 42; end program;"
        ));
        assert!(!errors.is_empty(), "{bad}");
        assert_eq!(program.body.len(), 2, "{bad}: {errors:?}");
        assert!(matches!(program.body[1], Stmt::Assign { .. }));
    }
    let (_, errors) = parse_expression("A < Next() < C");
    let error = errors[0].as_parser_error().unwrap();
    assert!(error.help.as_deref().unwrap().contains("once"));
}

#[test]
fn shift_words_are_ordinary_identifiers_and_partial_expressions_terminate() {
    let (_, errors) = parse(
        r#"program T; const shl: integer := 1; const SHR: integer := shl; begin null; end program;"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    for bad in ["not", "A and", "A <", "A shl", "not (", "A and or B"] {
        assert!(!parse_expression(bad).1.is_empty(), "{bad}");
    }
    let nested = format!("{}true", "not ".repeat(1000));
    assert!(!parse_expression(&nested).1.is_empty());
}
