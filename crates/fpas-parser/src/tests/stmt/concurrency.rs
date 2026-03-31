use super::super::{parse_ok, parse_with_errors};
use super::body_stmts;
use crate::ast::*;

#[test]
fn go_statement_parses_call_expression() {
    let stmts = body_stmts("program T; begin go Worker() end.");
    match &stmts[0] {
        Stmt::Go { expr, .. } => assert!(matches!(expr, Expr::Call { .. })),
        _ => panic!("expected Go statement"),
    }
}

#[test]
fn select_parses_arms_and_default() {
    let stmts = body_stmts(
        "program T; begin select case Value: integer from Ch: WriteLn(Value); default: begin WriteLn('none'); WriteLn('idle') end end end.",
    );

    match &stmts[0] {
        Stmt::Select {
            arms, default_body, ..
        } => {
            assert_eq!(arms.len(), 1);
            assert!(matches!(arms[0].body, Stmt::Call { .. }));
            assert_eq!(default_body.as_ref().map(Vec::len), Some(1));
            assert!(matches!(
                default_body.as_ref().unwrap()[0],
                Stmt::Block(_, _)
            ));
        }
        _ => panic!("expected Select statement"),
    }
}

#[test]
fn return_can_start_with_go_expression() {
    let program = parse_ok(
        "\
program T;

function Worker(): integer;
begin
  return 1
end;

function Spawn(): task;
begin
  return go Worker()
end;

begin
end.",
    );

    let Decl::Function(spawn) = &program.declarations[1] else {
        panic!("expected function declaration");
    };
    let FuncBody::Block { stmts, .. } = &spawn.body;
    match &stmts[0] {
        Stmt::Return(Some(Expr::Go(inner, _)), _) => {
            assert!(matches!(inner.as_ref(), Expr::Call { .. }));
        }
        _ => panic!("expected return go expression"),
    }
}

#[test]
fn select_parses_multiple_arms_different_types() {
    let stmts = body_stmts(
        "program T; begin select case Msg: string from Ch1: WriteLn(Msg); case Num: integer from Ch2: WriteLn(Num); end end.",
    );

    match &stmts[0] {
        Stmt::Select {
            arms, default_body, ..
        } => {
            assert_eq!(arms.len(), 2);
            assert_eq!(arms[0].binding, "Msg");
            assert_eq!(arms[1].binding, "Num");
            assert!(default_body.is_none());
        }
        _ => panic!("expected Select statement"),
    }
}

#[test]
fn select_parses_default_only() {
    let stmts = body_stmts("program T; begin select default: WriteLn('idle') end end.");

    match &stmts[0] {
        Stmt::Select {
            arms, default_body, ..
        } => {
            assert!(arms.is_empty());
            assert!(default_body.is_some());
        }
        _ => panic!("expected Select statement"),
    }
}

#[test]
fn empty_select_reports_error() {
    let (_, errs) = parse_with_errors("program T; begin select end end.");

    assert!(
        errs.iter()
            .any(|err| format!("{err:?}").contains("Empty `select`")),
        "expected empty select diagnostic, got: {errs:#?}"
    );
}

#[test]
fn go_as_expression_in_var_decl() {
    let stmts = body_stmts(
        "program T; function Work(): integer; begin return 1 end; begin var T: task := go Work() end.",
    );

    match &stmts[0] {
        Stmt::Var(def) => {
            assert!(matches!(def.value, Expr::Go(_, _)));
        }
        _ => panic!("expected var with go expression, got {:?}", stmts[0]),
    }
}
