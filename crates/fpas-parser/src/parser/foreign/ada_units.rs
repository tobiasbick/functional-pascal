//! Ada compilation-unit structure: `with`/`use` context clauses, the main `procedure Main is`
//! and `package` / `package body` units.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/program-structure/units.md`

use super::super::Parser;
use crate::ast::{QualifiedId, Unit};
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_lexer::Token;

/// Lowercase Ada library unit prefix and the FPAS import replacing it; `None` means the feature
/// is built in and needs no import. Longer prefixes come first.
const ADA_LIBRARY_UNITS: &[(&str, Option<&str>)] = &[
    (
        "ada.numerics.float_random",
        Some("Std.Math.Random as Random"),
    ),
    (
        "ada.numerics.discrete_random",
        Some("Std.Math.Random as Random"),
    ),
    ("ada.text_io", Some("Std.Console as Console")),
    ("ada.integer_text_io", Some("Std.Console as Console")),
    ("ada.float_text_io", Some("Std.Console as Console")),
    ("ada.numerics", Some("Std.Math as Math")),
    ("ada.calendar", Some("Std.Time as Time")),
    ("ada.command_line", Some("Std.Os.Args as Args")),
    ("ada.directories", Some("Std.Io.Fs as Fs")),
    ("ada.environment_variables", Some("Std.Os.Env as Env")),
    ("ada.strings", None),
    ("ada.characters", None),
    ("ada.containers", None),
];

impl Parser {
    /// Skips Ada `with` and `use` clauses before the compilation-unit header and reports them
    /// once with the matching `uses` clause.
    pub(in crate::parser) fn skip_ada_context_clauses(&mut self) {
        let start = self.current_span();
        let mut imports = Vec::<String>::new();
        while self.check(&Token::With) || Self::is_ident(self.current_token(), "use") {
            let is_with = self.eat(&Token::With);
            if !is_with {
                self.advance();
            }
            loop {
                let name = self.parse_qualified_id();
                if is_with
                    && let Some(import) = fpas_import(&name)
                    && !imports.contains(&import)
                {
                    imports.push(import);
                }
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.eat(&Token::Semicolon);
        }
        if self.pos == 0 || start.offset == self.current_span().offset {
            return;
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = if imports.is_empty() {
            "no `uses` clause: strings, arrays and dictionaries are built-in types".to_owned()
        } else {
            format!("uses {};", imports.join(", "))
        };
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada `with` and `use` context clauses are not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "After the `program` or `unit` header, `uses` imports each unit with an alias; `Std` units replace the Ada library.",
            },
            written,
            None,
        );
    }

    /// True at an Ada main program `procedure Name is` that starts a source file.
    pub(in crate::parser) fn at_ada_main_procedure(&self) -> bool {
        self.check(&Token::Procedure)
            && matches!(self.peek_token(), Token::Ident(_))
            && matches!(self.token_ahead(2), Token::Is)
    }

    /// True at an Ada `package` or `package body` unit header.
    pub(in crate::parser) fn at_ada_package(&self) -> bool {
        Self::is_ident(self.current_token(), "package")
            && matches!(self.peek_token(), Token::Ident(_))
    }

    /// Parses an Ada package as an FPAS unit and reports its structure once.
    ///
    /// A package body contributes its declarations; a package specification is skipped,
    /// because its routine declarations have no bodies.
    pub(in crate::parser) fn parse_ada_package(&mut self) -> Unit {
        let start = self.current_span();
        let (name, is_body) = self.eat_ada_package_header();
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let unit_name = name.parts.join(".");
        let replacement = format!("unit {unit_name}; … end unit;");
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada packages are not FPAS units",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "An FPAS unit has no separate specification and body: it is one declaration list where each export is marked `public`.",
            },
            written,
            None,
        );
        let declarations = if is_body {
            self.parse_declarations(true)
        } else {
            self.skip_ada_package_specification(&name);
            if self.at_ada_package() {
                let (_, is_body) = self.eat_ada_package_header();
                if is_body {
                    self.parse_declarations(true)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        self.eat_ada_named_end(&name);
        Unit {
            name,
            uses: Vec::new(),
            declarations,
            span: self.span_from(start),
        }
    }

    // `package [body] Name is`; returns the name and whether it is a body.
    fn eat_ada_package_header(&mut self) -> (QualifiedId, bool) {
        self.advance();
        let is_body = self.eat_ident("body");
        let name = self.parse_qualified_id();
        self.eat(&Token::Is);
        (name, is_body)
    }

    fn skip_ada_package_specification(&mut self, name: &QualifiedId) {
        while !self.at_end() && !self.at_ada_named_end(name) {
            self.advance();
        }
        self.eat_ada_named_end(name);
    }

    // `end Name;`, where `Name` is the last segment or the full dotted name.
    fn at_ada_named_end(&self, name: &QualifiedId) -> bool {
        let last = name.parts.last().map(String::as_str).unwrap_or_default();
        self.check(&Token::End)
            && matches!(self.peek_token(), Token::Ident(word)
                if word.eq_ignore_ascii_case(last)
                    || name.parts.first().is_some_and(|first| word.eq_ignore_ascii_case(first)))
    }

    fn eat_ada_named_end(&mut self, name: &QualifiedId) {
        if self.at_ada_named_end(name) {
            self.advance();
            self.parse_qualified_id();
            self.eat(&Token::Semicolon);
        }
    }

    /// Returns the token `n` positions after the current one, or `Eof`.
    pub(in crate::parser) fn token_ahead(&self, n: usize) -> &Token {
        let index = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[index].token
    }
}

fn fpas_import(name: &QualifiedId) -> Option<String> {
    let lower = name.parts.join(".").to_ascii_lowercase();
    if let Some((_, import)) = ADA_LIBRARY_UNITS
        .iter()
        .find(|(prefix, _)| lower == *prefix || lower.starts_with(&format!("{prefix}.")))
    {
        return import.map(str::to_owned);
    }
    if lower == "ada" || lower.starts_with("ada.") || lower.starts_with("gnat.") {
        return None;
    }
    let alias = name.parts.last()?;
    Some(format!("{} as {alias}", name.parts.join(".")))
}
