use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_primary(&mut self) -> Expr {
        match self.current_token().clone() {
            Token::Integer(v) => {
                let span = self.current_span();
                self.advance();
                Expr::Integer(v, span)
            }
            Token::Real(v) => {
                let span = self.current_span();
                self.advance();
                Expr::Real(v, span)
            }
            Token::Str(s) => {
                let span = self.current_span();
                self.advance();
                Expr::Str(s, span)
            }
            Token::True => {
                let span = self.current_span();
                self.advance();
                Expr::Bool(true, span)
            }
            Token::False => {
                let span = self.current_span();
                self.advance();
                Expr::Bool(false, span)
            }
            Token::LParen => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_expression();
                self.expect(&Token::RParen);
                Expr::Paren(Box::new(expr), self.span_from(start))
            }
            Token::LBracket => self.parse_array_or_dict_literal(),
            Token::Record => self.parse_record_literal(),
            Token::Ident(_) => self.parse_designator_or_call(),
            Token::Ok => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let inner = self.parse_expression();
                self.expect(&Token::RParen);
                Expr::ResultOk(Box::new(inner), self.span_from(start))
            }
            Token::Error => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let inner = self.parse_expression();
                self.expect(&Token::RParen);
                Expr::ResultError(Box::new(inner), self.span_from(start))
            }
            Token::Some => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let inner = self.parse_expression();
                self.expect(&Token::RParen);
                Expr::OptionSome(Box::new(inner), self.span_from(start))
            }
            Token::None => {
                let span = self.current_span();
                self.advance();
                Expr::OptionNone(span)
            }
            Token::Try => {
                let start = self.current_span();
                self.advance();
                let inner = self.parse_primary();
                Expr::Try(Box::new(inner), self.span_from(start))
            }
            Token::Go => {
                let start = self.current_span();
                self.advance();
                let inner = self.parse_primary();
                Expr::Go(Box::new(inner), self.span_from(start))
            }
            Token::Function => self.parse_function_expr(),
            _ => {
                let span = self.current_span();
                self.error_with_code(
                    PARSE_EXPECTED_EXPRESSION,
                    &format!(
                        "Expected expression, found `{}`",
                        super::super::token_display(self.current_token()),
                    ),
                    "An expression (value, variable, or function call) is required here.",
                    span,
                );
                self.advance();
                Expr::Integer(0, span)
            }
        }
    }

    fn parse_array_or_dict_literal(&mut self) -> Expr {
        let start = self.current_span();
        self.advance(); // consume '['

        // Empty array: []
        if self.check(&Token::RBracket) {
            self.advance();
            return Expr::ArrayLiteral(Vec::new(), self.span_from(start));
        }

        // Empty dict: [:]
        if self.check(&Token::Colon) {
            self.advance();
            self.expect(&Token::RBracket);
            return Expr::DictLiteral(Vec::new(), self.span_from(start));
        }

        // Parse the first expression
        let first = self.parse_expression();

        // If followed by ':', this is a dict literal
        if self.eat(&Token::Colon) {
            let first_value = self.parse_expression();
            let mut pairs = vec![(first, first_value)];
            while self.eat(&Token::Comma) {
                let key = self.parse_expression();
                self.expect(&Token::Colon);
                let value = self.parse_expression();
                pairs.push((key, value));
            }
            self.expect(&Token::RBracket);
            return Expr::DictLiteral(pairs, self.span_from(start));
        }

        // Otherwise it's a regular array literal
        let mut elements = vec![first];
        while self.eat(&Token::Comma) {
            elements.push(self.parse_expression());
        }
        self.expect(&Token::RBracket);
        Expr::ArrayLiteral(elements, self.span_from(start))
    }

    fn parse_record_literal(&mut self) -> Expr {
        let start = self.current_span();
        self.advance();
        let mut fields = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            let field_start = self.current_span();
            let (name, _) = self
                .expect_ident()
                .unwrap_or(("_error_".into(), field_start));
            self.expect(&Token::ColonAssign);
            let value = self.parse_expression();
            self.expect_semi();
            fields.push(FieldInit {
                name,
                value,
                span: self.span_from(field_start),
            });
        }
        self.expect(&Token::End);
        Expr::RecordLiteral {
            fields,
            span: self.span_from(start),
        }
    }

    /// Parse an anonymous function expression (lambda / closure).
    ///
    /// Syntax: `function(Params): ReturnType begin Stmts end`
    ///
    /// **Documentation:** `docs/pascal/04-functions.md`
    fn parse_function_expr(&mut self) -> Expr {
        let start = self.current_span();
        self.advance(); // consume `function`
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list();
        self.expect(&Token::RParen);
        self.expect(&Token::Colon);
        let return_type = self.parse_type_expr();

        let nested = self.parse_nested_decls();
        self.expect(&Token::Begin);
        let stmts = self.parse_statement_list();
        self.expect(&Token::End);

        Expr::Function {
            params,
            return_type,
            body: FuncBody::Block { nested, stmts },
            span: self.span_from(start),
        }
    }
}
