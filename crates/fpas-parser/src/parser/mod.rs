mod core;
mod decl;
mod display;
mod expr;
mod nesting;
mod program;
mod stmt;

use crate::error::ParseError;
use fpas_lexer::SpannedToken;

use display::token_display;

/// Placeholder identifier inserted when parsing cannot recover a real name.
pub(crate) const ERROR_IDENT: &str = "_error_";
#[cfg(test)]
pub(crate) use nesting::MAX_PARSER_NESTING_DEPTH;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    errors: Vec<ParseError>,
    nesting_depth: usize,
    nesting_limit_reached: bool,
}
