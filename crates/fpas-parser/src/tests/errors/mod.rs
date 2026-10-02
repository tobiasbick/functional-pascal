use super::{parse_compilation_unit_with_errors, parse_with_errors};
use crate::ParseDiagnostic;

mod api;
mod chained_comparison;
mod delimiters;
mod diagnostics;
mod keywords;
mod nesting;
mod recovery;
mod statement_separators;
mod syntax;
mod synthetic_eof;
mod trailing_input;
mod uses;

fn first_branch_statement(statement: &crate::Stmt) -> &crate::Stmt {
    let crate::Stmt::StatementList(statements, _) = statement else {
        panic!("expected statement list");
    };
    &statements[0]
}
