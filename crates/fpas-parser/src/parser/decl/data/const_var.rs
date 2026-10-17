//! Constant and variable declarations, one binding per keyword.
//!
//! Pascal sections and other foreign forms are parsed through
//! [`Parser::parse_binding_item`] and reported once per group.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`

use crate::ast::*;
use crate::parser::Parser;
use crate::parser::foreign::{BindingContext, BindingKeyword};
use fpas_diagnostics::codes::PARSE_EXPECTED_IDENTIFIER;
use fpas_lexer::Token;

impl Parser {
    /// Parses exactly one constant after its own `const` keyword.
    pub(in super::super) fn parse_const_declaration(
        &mut self,
        visibility: Visibility,
    ) -> Vec<Decl> {
        self.parse_binding_declaration(BindingKeyword::Const, visibility)
    }

    /// Parses one variable after its complete `var` prefix.
    pub(in super::super) fn parse_variable_declaration(
        &mut self,
        visibility: Visibility,
    ) -> Vec<Decl> {
        self.parse_binding_declaration(BindingKeyword::Var, visibility)
    }

    fn parse_binding_declaration(
        &mut self,
        keyword: BindingKeyword,
        visibility: Visibility,
    ) -> Vec<Decl> {
        let start = self.advance().span;
        if !self.can_start_declaration_definition() {
            let (kind, example) = match keyword {
                BindingKeyword::Const => ("constant", "const X: integer := 1;"),
                BindingKeyword::Var => ("variable", "var X: integer := 1;"),
            };
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                &format!("Expected a {kind} declaration after its keyword"),
                self.reserved_identifier_hint()
                    .unwrap_or(&format!("Add a declaration such as `{example}`.")),
                self.current_span(),
            );
            return Vec::new();
        }
        let items = self.parse_binding_items(keyword, true);
        self.report_binding_habits(
            start,
            &items,
            BindingContext::Keyword,
            visibility == Visibility::Public,
        );
        // Continuation items have no written `public`; they stay private.
        items
            .into_iter()
            .enumerate()
            .flat_map(|(index, item)| {
                let written = if index == 0 {
                    visibility
                } else {
                    Visibility::Private
                };
                Self::binding_declarations(item, written)
            })
            .collect()
    }

    /// Includes reserved block keywords so identifier recovery retains the rest of the definition.
    pub(super) fn can_start_declaration_definition(&self) -> bool {
        matches!(self.current_token(), Token::Ident(_)) || self.reserved_identifier_hint().is_some()
    }
}
