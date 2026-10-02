use super::super::body_stmts;
use crate::ast::*;

mod basic;
mod blocks;
mod nesting;

fn first(statement: &Stmt) -> &Stmt {
    let Stmt::StatementList(statements, _) = statement else {
        panic!("expected statement list");
    };
    &statements[0]
}
