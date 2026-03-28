use super::Lexer;
use crate::{Span, Token, error::lex_error};

impl Lexer<'_> {
    pub(super) fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub(super) fn current(&self) -> u8 {
        self.src[self.pos]
    }

    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.get(self.pos + offset).copied()
    }

    pub(super) fn advance(&mut self) -> u8 {
        let ch = self.src[self.pos];
        self.pos += 1;
        match ch {
            b'\n' => {
                self.line += 1;
                self.col = 1;
            }
            b'\r' => {
                if self.pos >= self.src.len() || self.src[self.pos] != b'\n' {
                    self.line += 1;
                    self.col = 1;
                }
            }
            _ => {
                self.col += 1;
            }
        }
        ch
    }

    pub(super) fn span_here(&self) -> (usize, u32, u32) {
        (self.pos, self.line, self.col)
    }

    pub(super) fn make_span(&self, start_offset: usize, start_line: u32, start_col: u32) -> Span {
        Span {
            offset: start_offset,
            length: self.pos - start_offset,
            line: start_line,
            column: start_col,
        }
    }

    pub(super) fn push_tok(&mut self, token: Token, so: usize, sl: u32, sc: u32) {
        let span = self.make_span(so, sl, sc);
        self.tokens.push(crate::SpannedToken { token, span });
    }

    pub(super) fn push_err(
        &mut self,
        code: fpas_diagnostics::DiagnosticCode,
        message: &str,
        hint: &str,
        so: usize,
        sl: u32,
        sc: u32,
    ) {
        let span = self.make_span(so, sl, sc);
        self.errors.push(lex_error(code, message, hint, span));
    }

    pub(super) fn emit_single(&mut self, token: Token) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.push_tok(token, so, sl, sc);
    }
}
