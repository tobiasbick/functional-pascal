mod core;
mod decl;
mod display;
mod expr;
mod program;
mod stmt;

use crate::error::ParseError;
use fpas_lexer::SpannedToken;

use display::token_display;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    errors: Vec<ParseError>,
}
