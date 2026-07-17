use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    pub(in crate::parser) fn parse_primary(&mut self) -> Expr {
        let start = self.current_span();
        let atom = self.parse_primary_atom();
        self.apply_postfix_suffixes(atom, start)
    }

    /// Parse a primary atom without postfix suffixes.
    ///
    /// Identifier paths become [`Expr::Designator`] or [`Expr::Call`]. Remaining
    /// `.Field` / `.Method(args)` / `[index]` suffixes are applied by
    /// [`Self::apply_postfix_suffixes`].
    fn parse_primary_atom(&mut self) -> Expr {
        // Avoid cloning the String payload when dispatching an identifier.
        if self.is_ident_designator_start() {
            return self.parse_designator_or_call();
        }

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
            Token::Ok => {
                let (inner, span) = self.parse_paren_wrapped_after_keyword();
                Expr::ResultOk(Box::new(inner), span)
            }
            Token::Error => {
                let (inner, span) = self.parse_paren_wrapped_after_keyword();
                Expr::ResultError(Box::new(inner), span)
            }
            Token::Some => {
                let (inner, span) = self.parse_paren_wrapped_after_keyword();
                Expr::OptionSome(Box::new(inner), span)
            }
            Token::None => {
                let span = self.current_span();
                self.advance();
                Expr::OptionNone(span)
            }
            Token::Nil => {
                let span = self.current_span();
                self.advance();
                Expr::Nil(span)
            }
            Token::Go => {
                let start = self.current_span();
                self.advance();
                let inner = self.parse_go_call_expression(start);
                Expr::Go(Box::new(inner), self.span_from(start))
            }
            Token::Function | Token::Procedure if self.at_closure_expr_start() => {
                self.parse_closure_expr()
            }
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
                Expr::Error(span)
            }
        }
    }

    fn parse_paren_wrapped_after_keyword(&mut self) -> (Expr, fpas_lexer::Span) {
        let start = self.current_span();
        self.advance();
        self.expect(&Token::LParen);
        let inner = self.parse_expression();
        self.expect(&Token::RParen);
        (inner, self.span_from(start))
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
        let fields = self.parse_field_init_list();
        Expr::RecordLiteral {
            fields,
            span: self.span_from(start),
        }
    }

    /// Parse `Field := Value;` initializers until `end`, then consume `end`.
    ///
    /// Shared by record literals, `new` expressions, and record update expressions.
    fn parse_field_init_list(&mut self) -> Vec<FieldInit> {
        let mut fields = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            let field_start = self.current_span();
            let (name, _) = self
                .expect_ident()
                .unwrap_or_else(|| self.error_ident(field_start));
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
        fields
    }

    /// Parse a record update expression: `base with Field := Value; … end`.
    ///
    /// The `with` token has already been peeked but **not consumed** when this is called.
    /// Consumes `with`, the field overrides, and `end`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
    pub(super) fn parse_record_update(&mut self, base: Expr, start: fpas_lexer::Span) -> Expr {
        self.advance(); // consume `with`
        let fields = self.parse_field_init_list();
        Expr::RecordUpdate {
            base: Box::new(base),
            fields,
            span: self.span_from(start),
        }
    }
}
