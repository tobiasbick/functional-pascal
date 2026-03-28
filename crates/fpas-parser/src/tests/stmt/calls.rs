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
