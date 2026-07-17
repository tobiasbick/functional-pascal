use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_designator_or_call(&mut self) -> Expr {
        let start = self.current_span();
        let designator = self.parse_designator();

        if self.check(&Token::LParen) {
            self.advance();
            let args = if self.check(&Token::RParen) {
                Vec::new()
            } else {
                self.parse_arg_list()
            };
            self.expect(&Token::RParen);
            Expr::Call {
                designator,
                args,
                span: self.span_from(start),
            }
        } else {
            Expr::Designator(designator)
        }
    }

    /// Apply zero or more `.Field` / `.Method(args)` / `[index]` suffixes to a primary atom.
    ///
    /// Returns `base` unchanged when no suffix is present. Never emits an empty operation list.
    ///
    /// **Documentation:** `docs/pascal/language/functions/README.md`
    pub(super) fn apply_postfix_suffixes(&mut self, base: Expr, start: fpas_lexer::Span) -> Expr {
        let mut operations = Vec::new();

        loop {
            if self.check(&Token::Dot) {
                let op_start = self.current_span();
                self.advance();
                let (name, _) = self
                    .expect_ident_after_dot()
                    .unwrap_or_else(|| self.error_ident(self.current_span()));

                if self.check(&Token::LParen) {
                    self.advance();
                    let args = if self.check(&Token::RParen) {
                        Vec::new()
                    } else {
                        self.parse_arg_list()
                    };
                    self.expect(&Token::RParen);
                    operations.push(PostfixOperation::MethodCall {
                        name,
                        args,
                        span: self.span_from(op_start),
                    });
                } else {
                    operations.push(PostfixOperation::Field {
                        name,
                        span: self.span_from(op_start),
                    });
                }
            } else if self.check(&Token::LBracket) {
                let op_start = self.current_span();
                self.advance();
                let index = self.parse_expression();
                self.expect(&Token::RBracket);
                operations.push(PostfixOperation::Index {
                    index: Box::new(index),
                    span: self.span_from(op_start),
                });
            } else {
                break;
            }
        }

        if operations.is_empty() {
            base
        } else {
            Expr::Postfix {
                base: Box::new(base),
                operations,
                span: self.span_from(start),
            }
        }
    }
}
