//! Lexer support for scanning `{$...}` compiler directives into [`Token::Directive`].
//!
//! **Documentation:** `docs/pascal/12-compiler-directives.md`

use super::Lexer;
use crate::Token;
use fpas_diagnostics::codes::LEX_UNTERMINATED_BRACE_COMMENT;

impl Lexer<'_> {
    /// Scans a compiler directive of the form `{$content}` and emits
    /// [`Token::Directive`] containing the trimmed content after the `$`.
    ///
    /// On an unterminated directive (no closing `}`) an error is pushed instead.
    pub(super) fn scan_directive(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance(); // consume '{'
        self.advance(); // consume '$'

        let start = self.pos;
        while !self.at_end() {
            if self.current() == b'}' {
                let raw = &self.src[start..self.pos];
                // SAFETY: `self.src` is a byte view of a `&str`; any contiguous
                // slice of it is valid UTF-8.
                let content: Box<str> = std::str::from_utf8(raw)
                    .expect("directive content is valid UTF-8: lexer source comes from &str")
                    .trim()
                    .into();
                self.advance(); // consume '}'
                self.push_tok(Token::Directive(content), so, sl, sc);
                return;
            }
            self.advance();
        }

        self.push_err(
            LEX_UNTERMINATED_BRACE_COMMENT,
            "Unterminated compiler directive starting with `{$`",
            "Add a closing `}` before end of file. Example: `{$IFDEF DEBUG}` … `{$ENDIF}`.",
            so,
            sl,
            sc,
        );
    }
}
