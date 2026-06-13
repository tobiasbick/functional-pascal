use crate::ast::*;
use crate::parser::Parser;
use fpas_lexer::Token;

impl Parser {
    pub(in super::super) fn parse_const_block(&mut self, visibility: Visibility) -> Vec<Decl> {
        self.advance();
        let mut defs = Vec::new();
        while let Token::Ident(_) = self.current_token() {
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
        while let Token::Ident(_) = self.current_token() {
            let var_def = self.parse_var_def(visibility);
            if mutable {
                defs.push(Decl::MutableVar(var_def));
            } else {
                defs.push(Decl::Var(var_def));
            }
        }
        defs
    }

    pub(in crate::parser) fn parse_typed_init_fields(
        &mut self,
        start: fpas_lexer::Span,
    ) -> (String, TypeExpr, Expr) {
        let (name, _) = self.expect_ident_or_error(start);
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        self.expect(&Token::ColonAssign);
        let value = self.parse_expression();
        (name, type_expr, value)
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
