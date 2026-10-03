use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::{PARSE_EXPECTED_IDENTIFIER, PARSE_EXPECTED_TOKEN};
use fpas_lexer::Token;

impl Parser {
    pub(in super::super) fn parse_const_block(&mut self, visibility: Visibility) -> Vec<Decl> {
        self.advance();
        let mut defs = Vec::new();
        if !self.at_declaration_name() {
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                "Expected a constant declaration after `const`",
                "Add a declaration such as `const X: integer := 1;`.",
                self.current_span(),
            );
        }
        if let Token::Ident(_) = self.current_token() {
            defs.push(Decl::Const(self.parse_const_def(visibility)));
        }
        defs
    }

    fn parse_const_def(&mut self, visibility: Visibility) -> ConstDef {
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

    pub(in super::super) fn parse_var_block(
        &mut self,
        mutable: bool,
        visibility: Visibility,
    ) -> Vec<Decl> {
        if mutable {
            self.advance();
        }
        self.advance();
        let mut defs = Vec::new();
        if !self.at_declaration_name() {
            let (kind, example) = if mutable {
                ("mutable variable", "mutable var X: integer := 1;")
            } else {
                ("variable", "var X: integer := 1;")
            };
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                &format!("Expected a {kind} declaration after the section keyword"),
                &format!("Add a declaration such as `{example}`."),
                self.current_span(),
            );
        }
        if let Token::Ident(_) = self.current_token() {
            let var_def = self.parse_var_def(visibility);
            if mutable {
                defs.push(Decl::MutableVar(var_def));
            } else {
                defs.push(Decl::Var(var_def));
            }
        }
        defs
    }

    fn at_declaration_name(&self) -> bool {
        matches!(self.current_token(), Token::Ident(_))
    }

    pub(in crate::parser) fn parse_typed_init_fields(
        &mut self,
        start: fpas_lexer::Span,
    ) -> (String, TypeExpr, Expr) {
        let (name, _) = self.expect_ident_or_error(start);
        self.reject_grouped_names(&name, |name| format!("var {name}: integer := 0;"), " ");
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        self.expect(&Token::ColonAssign);
        let value = self.parse_expression();
        (name, type_expr, value)
    }

    /// Diagnoses `A, B: T` and skips the extra names so the type still parses.
    ///
    /// `example` renders one corrected declaration per name; `separator` joins them.
    pub(in crate::parser) fn reject_grouped_names(
        &mut self,
        first: &str,
        example: fn(&str) -> String,
        separator: &str,
    ) {
        if !self.check(&Token::Comma) {
            return;
        }
        let span = self.current_span();
        let mut names = vec![first.to_owned()];
        while self.check(&Token::Comma) {
            self.advance();
            match self.current_token() {
                Token::Ident(name) => {
                    names.push(name.clone());
                    self.advance();
                }
                _ => break,
            }
        }
        let example = names
            .iter()
            .map(|name| example(name))
            .collect::<Vec<_>>()
            .join(separator);
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            "Each declaration names exactly one binding",
            &format!("Write one declaration per name, for example `{example}`."),
            span,
        );
    }

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
