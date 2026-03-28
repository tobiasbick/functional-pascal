use super::super::parse_with_errors;
use super::body_stmts;
use crate::ParseDiagnostic;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TO_OR_DOWNTO;

#[test]
fn for_to() {
    let stmts = body_stmts("program T; begin for I: integer := 0 to 9 do X := I end.");
    match &stmts[0] {
        Stmt::For {
            var_name,
            direction,
            ..
        } => {
            assert_eq!(var_name, "I");
            assert_eq!(*direction, ForDirection::To);
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn for_downto() {
    let stmts = body_stmts("program T; begin for I: integer := 9 downto 0 do X := I end.");
    match &stmts[0] {
        Stmt::For { direction, .. } => {
            assert_eq!(*direction, ForDirection::Downto);
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn for_loop_invalid_direction_uses_direction_code() {
    let (_, errors) = parse_with_errors("program T; begin for I: integer := 0 9 do X := I end.");
    let error = errors
        .iter()
        .find_map(|diagnostic| match diagnostic {
            ParseDiagnostic::Parser(error) if error.code == PARSE_EXPECTED_TO_OR_DOWNTO => {
                Some(error)
            }
            _ => None,
        })
        .expect("expected parser direction diagnostic");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("to 10 do")),
        "expected concrete for-loop direction help text"
    );
}

#[test]
fn for_in() {
    let stmts = body_stmts("program T; begin for X: integer in Arr do Y := X end.");
    match &stmts[0] {
        Stmt::ForIn {
            var_name, iterable, ..
        } => {
            assert_eq!(var_name, "X");
            assert!(matches!(iterable, Expr::Designator(_)));
        }
        _ => panic!("expected ForIn"),
    }
}

#[test]
fn while_loop() {
    let stmts = body_stmts("program T; begin while X > 0 do X := X - 1 end.");
    assert!(matches!(&stmts[0], Stmt::While { .. }));
}

#[test]
fn repeat_until() {
    let stmts = body_stmts("program T; begin repeat X := X + 1 until X = 10 end.");
    match &stmts[0] {
        Stmt::Repeat { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("expected Repeat"),
    }
}
