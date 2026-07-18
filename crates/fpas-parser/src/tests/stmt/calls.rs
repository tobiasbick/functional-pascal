use super::body_stmts;
use crate::ast::*;

#[test]
fn call_no_args() {
    let stmts = body_stmts("program T; begin Foo() end.");
    match &stmts[0] {
        Stmt::Call { args, .. } => {
            assert!(args.is_empty());
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn call_with_args() {
    let stmts = body_stmts("program T; begin WriteLn('hello', 42) end.");
    match &stmts[0] {
        Stmt::Call { args, .. } => {
            assert_eq!(args.len(), 2);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call() {
    let stmts = body_stmts("program T; begin Std.Console.WriteLn('hello') end.");
    match &stmts[0] {
        Stmt::Call { designator, .. } => {
            assert_eq!(designator.parts.len(), 3);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn postfix_method_chain_is_expression_statement() {
    let stmts = body_stmts("program T; begin Factory.Create().Destroy() end.");
    match &stmts[0] {
        Stmt::Expression {
            expr: Expr::Postfix { operations, .. },
            ..
        } => {
            assert_eq!(operations.len(), 1);
            assert!(matches!(
                &operations[0],
                PostfixOperation::MethodCall { name, .. } if name == "Destroy"
            ));
        }
        other => panic!("expected postfix expression statement, got {other:?}"),
    }
}

#[test]
fn parenthesized_postfix_method_chain_is_expression_statement() {
    let stmts = body_stmts("program T; begin (Factory.Create()).Destroy() end.");
    assert!(matches!(
        &stmts[0],
        Stmt::Expression {
            expr: Expr::Postfix { .. },
            ..
        }
    ));
}
