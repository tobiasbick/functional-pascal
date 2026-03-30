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
                // SAFETY: the lexer input is valid UTF-8 (it came from a &str); any
                // non-ASCII bytes inside the directive are still valid UTF-8 sequences
                // because the outer source did not trigger an error at that point.
                #[expect(
                    clippy::string_from_utf8_as_bytes,
                    reason = "we sliced a &[u8] view of a &str — round-tripping through from_utf8 is safe"
                )]
                let content: Box<str> = String::from_utf8_lossy(raw).trim().into();
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
