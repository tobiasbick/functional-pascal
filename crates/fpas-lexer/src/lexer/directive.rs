//! Lexer handling for a `{` immediately followed by `$`: the sequence is rejected.
//!
//! **Documentation:** `docs/pascal/program-structure/projects.md` (multi-file projects and `uses`)

use super::Lexer;
use fpas_diagnostics::codes::LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED;

impl Lexer<'_> {
    /// Scans `{$...}` through the closing `}` and reports a lexer error (no token is emitted).
    ///
    pub(super) fn scan_directive(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        while !self.at_end() {
            let closes = self.current() == b'}';
            self.advance();
            if closes {
                break;
            }
        }
        self.push_err(
            LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED,
            "`{$...}` is not valid source syntax",
            "Remove this sequence. Put shared declarations in another `.fpas` file and import the unit with `uses`.",
            so,
            sl,
            sc,
        );
    }
}
