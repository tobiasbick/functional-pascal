//! Diagnoses removed infix shifts without reserving their spellings as identifiers.
//!
//! Documentation: `docs/pascal/language/basics/operators.md`.

use super::super::Parser;
use crate::ast::Expr;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Rejects an infix shift tail after a complete expression and recovers its operands.
    pub(super) fn reject_retired_shifts(&mut self, expression: Expr) -> Expr {
        let start = expression.span();
        let mut rejected = false;
        while let Token::Ident(name) = self.current_token() {
            let replacement = if name.eq_ignore_ascii_case("shl") {
                "ShiftLeft"
            } else if name.eq_ignore_ascii_case("shr") {
                "ShiftRight"
            } else {
                break;
            };
            self.error_with_code(
                PARSE_EXPECTED_EXPRESSION,
                &format!("Infix `{name}` is not an operator"),
                &format!("Use `Std.Bits.{replacement}(Value, Count)` and import `uses Std.Bits;`."),
                self.current_span(),
            );
            self.advance();
            let _ = self.with_nesting(Self::parse_logical);
            rejected = true;
        }
        if rejected {
            Expr::Error(self.span_from(start))
        } else {
            expression
        }
    }
}
