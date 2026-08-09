//! Recovery for comment-like forms that are not valid Functional Pascal syntax.
//!
//! **Documentation:** `docs/pascal/language/basics/comments.md`

use super::Lexer;
use fpas_diagnostics::codes::LEX_INVALID_COMMENT_FORM;

const COMMENT_HINT: &str = "Use `// comment`. For multiple lines, prefix each line with `//`.";

impl Lexer<'_> {
    /// Consumes `{...}` or the remaining source and reports one actionable diagnostic.
    pub(super) fn scan_invalid_brace_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        while !self.at_end() {
            let closes = self.current() == b'}';
            self.advance();
            if closes {
                break;
            }
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`{...}` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
    }

    /// Consumes `(*...*)` or the remaining source and reports one actionable diagnostic.
    pub(super) fn scan_invalid_paren_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        while !self.at_end() {
            if self.current() == b'*' && self.peek_at(1) == Some(b')') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`(*...*)` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
    }
}
