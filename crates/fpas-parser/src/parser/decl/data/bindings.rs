//! `const` and `var` binding syntax with initializer-only local inference.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`.

use crate::ast::{BindingDef, Decl, Visibility};
use crate::parser::Parser;
use fpas_diagnostics::codes::{PARSE_EXPECTED_IDENTIFIER, PARSE_EXPECTED_TOKEN};
use fpas_lexer::Token;

impl Parser {
    /// Parse one explicitly typed unit/program binding.
    pub(in super::super) fn parse_binding_declaration(
        &mut self,
        mutable: bool,
        visibility: Visibility,
    ) -> Vec<Decl> {
        self.advance();
        if !matches!(self.current_token(), Token::Ident(_)) {
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                "Expected a binding name after the declaration keyword",
                "Write one initialized binding, for example `const Answer: integer := 42;`.",
                self.current_span(),
            );
            return Vec::new();
        }
        let definition = self.parse_binding_definition(false, mutable, visibility);
        self.expect_semi();
        vec![if mutable {
            Decl::Var(definition)
        } else {
            Decl::Const(definition)
        }]
    }

    /// Parse a binding payload; only local statements may omit their type.
    pub(in crate::parser) fn parse_binding_definition(
        &mut self,
        local: bool,
        mutable: bool,
        visibility: Visibility,
    ) -> BindingDef {
        let start = self.current_span();
        let (name, _) = self.expect_ident_or_error(start);
        let example: fn(&str) -> String = if mutable {
            |name| format!("var {name}: integer := 0;")
        } else {
            |name| format!("const {name}: integer := 0;")
        };
        self.reject_grouped_names(&name, example, " ");
        let type_expr = if self.eat(&Token::Colon) {
            Some(self.parse_type_expr())
        } else {
            if !local {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Unit/program bindings require an explicit type",
                    "Add an annotation, for example `const Answer: integer := Compute();`. Only local bindings may infer their type.",
                    start,
                );
            }
            None
        };
        self.expect(&Token::ColonAssign);
        let value = self.parse_expression();
        BindingDef {
            name,
            type_expr,
            value,
            visibility,
            span: self.span_from(start),
        }
    }

    /// Recover obsolete binding spelling with a concrete migration hint.
    pub(in crate::parser) fn reject_mutable_binding_keyword(&mut self) {
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            "The `mutable var` binding spelling is obsolete",
            "Write `var Value: integer := 0;` for writable storage, or `const Value := 0;` for an immutable local binding.",
            self.current_span(),
        );
        self.advance();
    }
}
