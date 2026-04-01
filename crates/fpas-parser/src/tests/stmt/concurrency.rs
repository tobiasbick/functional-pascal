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

