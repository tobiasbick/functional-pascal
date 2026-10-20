//! Pascal output width and precision specifiers in call arguments, such as `WriteLn(X:0:2)`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/std/console/output.md`

use super::super::Parser;
use crate::ast::Expr;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_diagnostics::{Applicability, ForeignHabit};
use fpas_lexer::Token;

impl Parser {
    /// Skips `:Width` and `:Width:Decimals` after a call argument and reports them once.
    pub(in crate::parser) fn skip_width_specifiers(&mut self, argument: &Expr) {
        if !self.check(&Token::Colon) {
            return;
        }
        let start = argument.span();
        while self.eat(&Token::Colon) {
            self.parse_expression();
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = self.source_text(start).to_owned();
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Pascal width specifier is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "FPAS output has no `:width` or `:decimals` syntax; format the text first, for example with `'%8d'.Format(N)` or `'%.2f'.Format(X)`.",
            },
            written,
            // Dropping the specifiers changes the printed text.
            Some(Applicability::NeedsReview),
        );
    }
}
