use super::{super::parse_with_errors, body_stmts};
use crate::ast::{Expr, Stmt};

#[test]
fn discard_accepts_values_postfix_and_go_operands() {
    let statements =
        body_stmts("program T; begin DisCard 42; discard Build().Value; discard go Worker(); end.");
    assert!(matches!(
        &statements[0],
        Stmt::Discard {
            expr: Expr::Integer(42, _),
            ..
        }
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Discard {
            expr: Expr::Postfix { .. },
            ..
        }
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Discard {
            expr: Expr::Go(_, _),
            ..
        }
    ));
}

#[test]
fn discard_requires_operand_and_terminator() {
    for source in [
        "program T; begin discard; end.",
        "program T; begin discard 1 end.",
    ] {
        assert!(!parse_with_errors(source).1.is_empty());
    }
}

#[test]
fn discard_identifier_has_rename_hint() {
    let (_, errors) = parse_with_errors("program T; var discard: integer := 1; begin end.");
    assert!(
        errors
            .iter()
            .filter_map(crate::ParseDiagnostic::as_parser_error)
            .any(|error| error
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("Rename")))
    );
}
