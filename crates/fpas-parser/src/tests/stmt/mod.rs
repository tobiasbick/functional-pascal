use super::parse_ok;
use crate::ast::*;

mod assignments;
mod blocks;
mod calls;
mod concurrency;
mod conditionals;
mod const_defs;
mod discard;
mod flow;
mod loops;
mod var_defs;

fn body_stmts(src: &str) -> Vec<Stmt> {
    parse_ok(src).body
}
