//! Main lexer driver: token dispatch, trivia skipping, and EOF handling.
//!
//! **Documentation:** `docs/pascal/getting-started/README.md` (lexical structure and keywords).

mod directive;
mod identifiers;
mod navigation;
mod numbers;
mod strings;
mod symbols;
mod trivia;

use crate::{LexError, SourceComment, SpannedToken, Token};
use fpas_diagnostics::codes::LEX_UNEXPECTED_CHARACTER;

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: u32,
    col: u32,
    source_id: u32,
    tokens: Vec<SpannedToken>,
    comments: Vec<SourceComment>,
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self::with_source_id(source, 0)
    }

    pub fn with_source_id(source: &'a str, source_id: u32) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            col: 1,
            source_id,
            tokens: Vec::new(),
            comments: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn tokenize_with_comments(
        mut self,
    ) -> (Vec<SpannedToken>, Vec<SourceComment>, Vec<LexError>) {
        self.scan_all();
        (self.tokens, self.comments, self.errors)
    }

    fn scan_all(&mut self) {
        loop {
            self.skip_trivia();
            if self.at_end() {
                let (so, sl, sc) = self.span_here();
                self.push_tok(Token::Eof, so, sl, sc);
                break;
            }
            self.scan_token();
        }
    }

    fn scan_token(&mut self) {
        match self.current() {
            b'{' if self.is_directive_after_brace() => self.scan_directive(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_ident_or_keyword(),
            b'0'..=b'9' => self.scan_number(),
            b'$' => self.scan_hex_integer(),
            b'\'' | b'#' => self.scan_string_literal(),
            b':' => self.scan_colon(),
            b'.' => self.scan_dot(),
            b'<' => self.scan_less(),
            b'>' => self.scan_greater(),
            b';' => self.emit_single(Token::Semicolon),
            b',' => self.emit_single(Token::Comma),
            b'(' => self.emit_single(Token::LParen),
            b')' => self.emit_single(Token::RParen),
            b'[' => self.emit_single(Token::LBracket),
            b']' => self.emit_single(Token::RBracket),
            b'+' => self.emit_single(Token::Plus),
            b'-' => self.emit_single(Token::Minus),
            b'*' => self.emit_single(Token::Star),
            b'/' => self.emit_single(Token::Slash),
            b'=' => self.emit_single(Token::Equal),
            _ if self.starts_non_ascii_identifier() => self.scan_invalid_identifier(),
            _ => {
                let (so, sl, sc) = self.span_here();
                let ch = self.advance_utf8_char();
                self.push_err(
                    LEX_UNEXPECTED_CHARACTER,
                    &format!("Unexpected character `{ch}`"),
                    "Remove this character or replace it with a valid Pascal token such as `:=`, `;`, `(`, or an identifier.",
                    so, sl, sc,
                );
            }
        }
    }
}
