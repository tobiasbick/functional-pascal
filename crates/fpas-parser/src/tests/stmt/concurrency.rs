use super::super::parse_ok;
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
    let FuncBody::Block { stmts, .. } = &spawn.body else {
        panic!("expected block body");
    };
    match &stmts[0] {
        Stmt::Return(Some(Expr::Go(inner, _)), _) => {
            assert!(matches!(inner.as_ref(), Expr::Call { .. }));
        }
        _ => panic!("expected return go expression"),
    }
}
