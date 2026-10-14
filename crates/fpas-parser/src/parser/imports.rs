//! Contextual `as` syntax in direct import lists.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use fpas_diagnostics::codes::PARSE_MISSING_IMPORT_ALIAS;
use fpas_lexer::Token;

use super::Parser;
use crate::ast::{Import, ImportAlias};

impl Parser {
    pub(super) fn parse_uses_clause(&mut self) -> Vec<Import> {
        self.advance();
        let mut imports = vec![self.parse_import()];
        while self.eat(&Token::Comma) {
            imports.push(self.parse_import());
        }
        self.expect_semi();
        imports
    }

    fn parse_import(&mut self) -> Import {
        let start = self.current_span();
        let unit = self.parse_qualified_id();
        let alias = if matches!(self.current_token(), Token::Ident(name) if name.eq_ignore_ascii_case("as"))
        {
            self.advance();
            let (name, span) = self
                .expect_ident()
                .unwrap_or_else(|| self.error_ident(self.current_span()));
            Some(ImportAlias { name, span })
        } else if unit.parts.iter().any(|part| part == "_error_") {
            None
        } else {
            let unit_name = unit.parts.join(".");
            let last = unit.parts.last().map(String::as_str).unwrap_or("Imported");
            let suggested = if unit.parts.len() == 1 || last.eq_ignore_ascii_case("std") {
                format!("{last}Unit")
            } else {
                last.to_owned()
            };
            self.error_with_code(
                PARSE_MISSING_IMPORT_ALIAS,
                &format!("Import `{unit_name}` requires an explicit alias"),
                &format!("Write `uses {unit_name} as {suggested};` and access public names through `{suggested}.Name`."),
                unit.span,
            );
            None
        };
        Import {
            unit,
            alias,
            span: self.span_from(start),
        }
    }
}
