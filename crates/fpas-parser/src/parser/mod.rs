mod block_closers;
mod core;
mod decl;
mod display;
mod expr;
mod foreign;
mod imports;
mod nesting;
mod patterns;
mod program;
mod stmt;

use crate::error::ParseError;
use fpas_lexer::{SpannedToken, Token};

use display::token_display;

/// Placeholder identifier inserted when parsing cannot recover a real name.
pub(crate) const ERROR_IDENT: &str = "_error_";
#[cfg(test)]
pub(crate) use nesting::MAX_PARSER_NESTING_DEPTH;

pub struct Parser {
    source: String,
    tokens: Vec<SpannedToken>,
    pos: usize,
    errors: Vec<ParseError>,
    nesting_depth: usize,
    nesting_limit_reached: bool,
    block_closers: Vec<Token>,
    /// Token position and type name of the initializer being parsed for a named declared type.
    initializer_type: Option<(usize, String)>,
    /// Error index and ending text of an Ada routine header whose fix still needs its
    /// `end Name` edit.
    ada_header_fix: Option<(usize, &'static str)>,
    /// The same for an Ada main program, whose `end Name;` becomes `end.`.
    ada_program_fix: Option<(usize, &'static str)>,
}
