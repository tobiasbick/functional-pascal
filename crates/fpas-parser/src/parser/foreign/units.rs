//! Delphi unit structure: `interface`/`implementation` sections, the `end.` unit ending and
//! Delphi run-time library units in `uses`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/program-structure/units.md`

use super::super::Parser;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::{PARSE_FOREIGN_SYNTAX, PARSE_MISSING_IMPORT_ALIAS};
use fpas_lexer::{Span, Token};

/// Lowercase Delphi run-time library unit, its FPAS replacement and why. One line per unit.
const DELPHI_RTL_UNITS: &[(&str, &str, &str)] = &[
    (
        "sysutils",
        "uses Std.Text.Parse as Parse, Std.Text.Format as Conv;",
        "Delphi `SysUtils` routines live in `Std` units such as `Std.Text.Parse`, `Std.Text.Format` and `Std.Io.Fs`.",
    ),
    (
        "strutils",
        "uses Std.Text.Format as Conv;",
        "String routines are native methods such as `Text.Trim()`; conversions live in `Std.Text.Format`.",
    ),
    (
        "math",
        "uses Std.Math as Math;",
        "Math routines live in `Std.Math`, imported with an alias.",
    ),
    (
        "dateutils",
        "uses Std.Time as Time;",
        "Date and time routines live in `Std.Time`, imported with an alias.",
    ),
    (
        "classes",
        "uses Std.Tasks as Tasks;",
        "Delphi `Classes` has no FPAS unit: lists are `array of T`, and threads are tasks from `Std.Tasks`.",
    ),
    (
        "generics.collections",
        "array of T",
        "Collections are the native types `array of T` and `dict of K to V`; remove the import.",
    ),
];

impl Parser {
    /// True at a Delphi `interface` section directly after the unit header.
    pub(in crate::parser) fn at_delphi_interface_section(&self) -> bool {
        Self::is_ident(self.current_token(), "interface")
    }

    /// Skips the Delphi `interface` section through `implementation` and reports both sections
    /// once; the implementation declarations are then parsed as the unit's declarations.
    pub(in crate::parser) fn skip_delphi_interface_section(&mut self, unit_name: &str) {
        let start = self.current_span();
        let mut first_header: Option<Span> = None;
        while !self.at_end() && !Self::is_ident(self.current_token(), "implementation") {
            if first_header.is_none()
                && matches!(self.current_token(), Token::Function | Token::Procedure)
            {
                first_header = Some(self.skip_routine_header());
                continue;
            }
            self.advance();
        }
        self.eat_ident("implementation");
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let declaration = first_header
            .map(|span| self.source_text(span).to_owned())
            .unwrap_or_else(|| "function Name(): T;".to_owned());
        let kind = if declaration
            .get(..9)
            .is_some_and(|word| word.eq_ignore_ascii_case("procedure"))
        {
            "procedure"
        } else {
            "function"
        };
        let replacement =
            format!("unit {unit_name}; public {declaration} begin … end {kind}; end unit;");
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Delphi `interface`/`implementation` sections are not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "An FPAS unit is one declaration list: mark each export `public` and close the unit with `end unit;`.",
            },
            written,
            None,
        );
    }

    /// Consumes the Delphi unit ending `end.` after a reported `interface` section.
    pub(in crate::parser) fn eat_delphi_unit_ending(&mut self) -> bool {
        if self.check(&Token::End) && self.peek_token() == &Token::Dot {
            self.advance();
            self.advance();
            true
        } else {
            false
        }
    }

    /// Reports a Delphi run-time library unit imported without alias, instead of asking for an
    /// alias of a unit that does not exist. Returns whether it was reported.
    pub(in crate::parser) fn report_delphi_rtl_import(
        &mut self,
        unit_name: &str,
        span: Span,
    ) -> bool {
        let lower = unit_name.to_ascii_lowercase();
        // Delphi XE and later qualify run-time units with `System.`.
        let lower = lower.strip_prefix("system.").unwrap_or(&lower);
        let Some((_, replacement, explanation)) =
            DELPHI_RTL_UNITS.iter().find(|(name, _, _)| *name == lower)
        else {
            return false;
        };
        self.report_foreign_habit(
            PARSE_MISSING_IMPORT_ALIAS,
            &format!("Delphi unit `{unit_name}` does not exist in FPAS"),
            ForeignHabit {
                found: unit_name,
                replacement,
                explanation,
            },
            span,
            // The replacement is a whole `uses` clause, not a unit name.
            None,
        );
        true
    }

    /// Skips a routine header through its terminating `;` and returns its span.
    fn skip_routine_header(&mut self) -> Span {
        let start = self.current_span();
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::LParen => depth += 1,
                Token::RParen => depth = depth.saturating_sub(1),
                Token::Semicolon if depth == 0 => {
                    self.advance();
                    break;
                }
                _ => {}
            }
            self.advance();
        }
        self.span_from(start)
    }

    pub(super) fn is_ident(token: &Token, word: &str) -> bool {
        matches!(token, Token::Ident(name) if name.eq_ignore_ascii_case(word))
    }

    pub(super) fn eat_ident(&mut self, word: &str) -> bool {
        if Self::is_ident(self.current_token(), word) {
            self.advance();
            true
        } else {
            false
        }
    }
}
