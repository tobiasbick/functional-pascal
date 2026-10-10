use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Parses a primary atom and its existing postfix suffixes.
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
            Token::Array => self.parse_native_factory(),
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
            Token::Record => self.reject_record_literal(),
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
                    self.reserved_identifier_hint().unwrap_or(
                        "An expression (value, variable, or function call) is required here.",
                    ),
                    span,
                );
                if !self.is_expression_recovery_boundary() {
                    self.advance();
                }
                Expr::Error(span)
            }
        }
    }

    /// Parses the keyword-owned `string.Chr` and `array.Fill` call shape.
    fn parse_native_factory(&mut self) -> Expr {
        let start = self.current_span();
        let owner = if self.check(&Token::Array) {
            "array"
        } else {
            "string"
        };
        self.advance();
        self.expect(&Token::Dot);
        let (name, name_span) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(self.current_span()));
        self.expect(&Token::LParen);
        let args = if self.check(&Token::RParen) {
            Vec::new()
        } else {
            self.parse_arg_list()
        };
        self.expect(&Token::RParen);
        let span = self.span_from(start);
        Expr::Call {
            designator: Designator {
                parts: vec![
                    DesignatorPart::Ident(owner.into(), start),
                    DesignatorPart::Ident(name, name_span),
                ],
                span,
            },
            args,
            span,
        }
    }

    fn parse_paren_wrapped_after_keyword(&mut self) -> (Expr, fpas_lexer::Span) {
        let start = self.current_span();
        self.advance();
        self.expect(&Token::LParen);
        // A named value is kept so semantic analysis can explain the positional-only form.
        let inner = self.parse_argument();
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
}
