//! Individual constant and variable declarations.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`

use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_EXPECTED_IDENTIFIER;
use fpas_lexer::Token;

impl Parser {
    /// Parses exactly one constant after its own `const` keyword.
    pub(in super::super) fn parse_const_declaration(
        &mut self,
        visibility: Visibility,
    ) -> Option<Decl> {
        self.advance();
        if !self.can_start_declaration_definition() {
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                "Expected a constant declaration after `const`",
                self.reserved_identifier_hint()
                    .unwrap_or("Add a declaration such as `const X: integer := 1;`."),
                self.current_span(),
            );
            return None;
        }
        Some(Decl::Const(self.parse_const_def(visibility)))
    }

    /// Parses a constant definition, including one recovered after a missing keyword.
    pub(in crate::parser::decl) fn parse_const_def(&mut self, visibility: Visibility) -> ConstDef {
        let start = self.current_span();
        let (name, type_expr, value) = self.parse_typed_init_fields(start);
        self.expect_semi();
        ConstDef {
            name,
            type_expr,
            value,
            visibility,
            span: self.span_from(start),
        }
    }

    /// Parses one variable after its complete `var` prefix.
    pub(in super::super) fn parse_variable_declaration(
        &mut self,
        visibility: Visibility,
    ) -> Option<Decl> {
        self.advance();
        if !self.can_start_declaration_definition() {
            let (kind, example) = ("variable", "var X: integer := 1;");
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                &format!("Expected a {kind} declaration after its keyword"),
                self.reserved_identifier_hint()
                    .unwrap_or(&format!("Add a declaration such as `{example}`.")),
                self.current_span(),
            );
            return None;
        }
        let definition = self.parse_var_def(visibility);
        Some(Decl::Var(definition))
    }

    /// Includes reserved block keywords so identifier recovery retains the rest of the definition.
    pub(super) fn can_start_declaration_definition(&self) -> bool {
        matches!(self.current_token(), Token::Ident(_)) || self.reserved_identifier_hint().is_some()
    }

    /// Parses a single binding's name, required type, and initializer.
    pub(in crate::parser) fn parse_typed_init_fields(
        &mut self,
        start: fpas_lexer::Span,
    ) -> (String, TypeExpr, Expr) {
        let (name, _) = self.expect_ident_or_error(start);
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        self.expect(&Token::ColonAssign);
        let value = self.parse_initializer(&type_expr);
        (name, type_expr, value)
    }

    /// Parses one variable definition and its terminator.
    pub(in crate::parser) fn parse_var_def(&mut self, visibility: Visibility) -> VarDef {
        let start = self.current_span();
        let (name, type_expr, value) = self.parse_typed_init_fields(start);
        self.expect_semi();
        VarDef {
            name,
            type_expr,
            value,
            visibility,
            span: self.span_from(start),
        }
    }
}
