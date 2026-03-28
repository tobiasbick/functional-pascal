use super::super::parse_with_errors;
use super::body_stmts;
use crate::ParseDiagnostic;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_INVALID_STATEMENT_START;

#[test]
fn return_with_value() {
    let stmts = body_stmts("program T; begin return 42 end.");
    match &stmts[0] {
        Stmt::Return(Some(expr), _) => {
            assert!(matches!(expr, Expr::Integer(42, _)));
        }
        _ => panic!("expected Return with value"),
    }
}

#[test]
fn return_bare() {
    let stmts = body_stmts("program T; begin return end.");
    match &stmts[0] {
        Stmt::Return(None, _) => {}
        _ => panic!("expected bare Return"),
    }
}

#[test]
fn panic_stmt() {
    let stmts = body_stmts("program T; begin panic('error') end.");
    assert!(matches!(&stmts[0], Stmt::Panic(_, _)));
}

#[test]
fn break_stmt() {
    let stmts = body_stmts("program T; begin break end.");
    assert!(matches!(&stmts[0], Stmt::Break(_)));
}

#[test]
fn continue_stmt() {
    let stmts = body_stmts("program T; begin continue end.");
    assert!(matches!(&stmts[0], Stmt::Continue(_)));
}

#[test]
fn invalid_statement_start_uses_statement_start_code() {
    let (_, errors) = parse_with_errors("program T; begin ; end.");
    let error = errors
        .iter()
        .find_map(|diagnostic| match diagnostic {
            ParseDiagnostic::Parser(error) => Some(error),
            ParseDiagnostic::Lexer(_) => None,
        })
        .expect("expected parser diagnostic");
    assert_eq!(error.code, PARSE_INVALID_STATEMENT_START);
}
