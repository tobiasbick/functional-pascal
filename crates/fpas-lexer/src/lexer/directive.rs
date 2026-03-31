//! Lexer handling for a `{` immediately followed by `$`: the sequence is rejected.
//!
//! **Documentation:** `docs/pascal/10-projects.md` (multi-file projects and `uses`)

use super::Lexer;
use fpas_diagnostics::codes::{
    LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED, LEX_UNTERMINATED_BRACE_COMMENT,
};

impl Lexer<'_> {
    /// Scans `{$...}` through the closing `}` and reports a lexer error (no token is emitted).
    ///
    /// On an unterminated sequence (no closing `}` before EOF) an unterminated-brace error is pushed.
    pub(super) fn scan_directive(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance(); // consume '{'
        self.advance(); // consume '$'

        while !self.at_end() {
            if self.current() == b'}' {
                self.advance(); // consume '}'
                self.push_err(
                    LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED,
                    "`{$...}` is not valid source syntax",
                    "Remove this sequence. Put shared declarations in another `.fpas` file and import the unit with `uses`.",
                    so,
                    sl,
                    sc,
                );
                return;
            }
            self.advance();
        }

        self.push_err(
            LEX_UNTERMINATED_BRACE_COMMENT,
            "Unterminated `{$...}` sequence starting with `{$`",
            "Add a closing `}` before end of file, or use a brace comment `{ ... }` without `$` after `{`.",
            so,
            sl,
            sc,
        );
    }
}
