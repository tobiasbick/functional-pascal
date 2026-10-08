//! Contextual `as` syntax in direct import lists.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

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
        } else {
            None
        };
        Import {
            unit,
            alias,
            span: self.span_from(start),
        }
    }
}
