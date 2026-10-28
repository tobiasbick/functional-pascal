//! Ada attributes such as `Integer'Image (X)`, `Items'Length` and `Items'Range`.
//!
//! An apostrophe directly after an identifier, followed by a name and no closing apostrophe, is
//! an attribute; FPAS strings never follow an identifier without a space or operator.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use crate::Token;
use crate::lexer::Lexer;
use fpas_diagnostics::{Applicability, Fix, SourceSpan, fix_title};

impl Lexer<'_> {
    /// True at `'Name` directly after an identifier.
    pub(in crate::lexer) fn at_ada_attribute(&self) -> bool {
        if self.current() != b'\'' {
            return false;
        }
        let follows_identifier = self.tokens.last().is_some_and(|token| {
            matches!(token.token, Token::Ident(_)) && token.end.offset == self.pos
        });
        if !follows_identifier {
            return false;
        }
        let name_length = self.attribute_name_length();
        name_length > 0 && self.peek_at(1 + name_length) != Some(b'\'')
    }

    /// Consumes `'Name` and reports the attribute with the FPAS form for common ones.
    pub(in crate::lexer) fn scan_ada_attribute(&mut self) {
        let prefix = match self.tokens.last().map(|token| &token.token) {
            Some(Token::Ident(name)) => name.clone(),
            _ => String::new(),
        };
        let start = self.span_here();
        let name_length = self.attribute_name_length();
        for _ in 0..=name_length {
            self.advance();
        }
        let attribute = self
            .source
            .get(start.0 + 1..self.pos)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let replacement = match (prefix.to_ascii_lowercase().as_str(), attribute.as_str()) {
            ("integer" | "natural" | "positive", "image") => "Conv.IntToStr(X)".to_owned(),
            ("float" | "long_float", "image") => "Conv.RealToStr(X)".to_owned(),
            (_, "length") => format!("{prefix}.Length()"),
            (_, "first") => "0".to_owned(),
            (_, "last") => format!("{prefix}.Length() - 1"),
            (_, "range") => format!("0 to {prefix}.Length() - 1"),
            _ => "a function or method call".to_owned(),
        };
        self.report_foreign_token(
            "Ada attribute is not FPAS syntax",
            &replacement,
            "FPAS has no attributes; use `Std.Text.Format` conversions, methods such as `Items.Length()`, and zero-based indexes.",
            start,
            None,
        );
        // `Items'Length` is exactly `Items.Length()`. Ada arrays may start at any index, so
        // the zero-based forms of `First`, `Last` and `Range` need review.
        let applicability = match attribute.as_str() {
            "length" => Some(Applicability::MachineApplicable),
            "first" | "last" | "range" => Some(Applicability::NeedsReview),
            _ => None,
        };
        let prefix_span = self.tokens.last().map(|token| token.span);
        if let (Some(applicability), Some(prefix_span), false) =
            (applicability, prefix_span, prefix.is_empty())
        {
            let written = self.make_span(prefix_span.offset, prefix_span.line, prefix_span.column);
            let fix = SourceSpan::try_from(written).ok().and_then(|span| {
                Fix::replace(fix_title(&replacement), applicability, span, replacement)
            });
            if let Some(diagnostic) = self.errors.pop() {
                self.errors.push(diagnostic.with_fix(fix));
            }
        }
    }

    fn attribute_name_length(&self) -> usize {
        let mut length = 0;
        while self
            .peek_at(1 + length)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            length += 1;
        }
        length
    }
}
