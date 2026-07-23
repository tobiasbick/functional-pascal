use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_comparison(&mut self) -> Expr {
        let start = self.current_span();
        let left = self.parse_additive();

        let op = match self.current_token() {
            Token::Equal => Some(BinaryOp::Eq),
            Token::NotEqual => Some(BinaryOp::NotEq),
            Token::Less => Some(BinaryOp::Lt),
            Token::Greater => Some(BinaryOp::Gt),
            Token::LessEqual => Some(BinaryOp::LtEq),
            Token::GreaterEqual => Some(BinaryOp::GtEq),
            Token::In => Some(BinaryOp::In),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let right = self.parse_additive();
            let expr = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            };
            self.recover_from_chained_comparison();
            expr
        } else {
            left
        }
    }

    /// Rejects `A op B op C` and skips the erroneous tail so statement parsing can continue.
    fn recover_from_chained_comparison(&mut self) {
        if !self.is_comparison_token() {
            return;
        }
        let span = self.current_span();
        self.error_with_code(
            PARSE_EXPECTED_EXPRESSION,
            "Chained comparison operators are not allowed",
            "Use at most one comparison operator per expression (for example `(A = B) and (C = D)`).",
            span,
        );
        while self.is_comparison_token() {
            self.advance();
            let _ = self.parse_additive();
        }
    }

    pub(super) fn parse_additive(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_multiplicative();

        loop {
            let op = match self.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                Token::Or => BinaryOp::Or,
                Token::Xor => BinaryOp::Xor,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        left
    }

    pub(super) fn parse_multiplicative(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_unary();

        loop {
            let op = match self.current_token() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::RealDiv,
                Token::Div => BinaryOp::IntDiv,
                Token::Mod => BinaryOp::Mod,
                Token::And => BinaryOp::And,
                Token::Shl => BinaryOp::Shl,
                Token::Shr => BinaryOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        left
    }

    pub(super) fn parse_unary(&mut self) -> Expr {
        let start = self.current_span();

        if self.check(&Token::Not) {
            self.advance();
            let operand = self.parse_unary();
            return Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                span: self.span_from(start),
            };
        }

        if self.check(&Token::Minus) {
            self.advance();
            let operand = self.parse_unary();
            return Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
                span: self.span_from(start),
            };
        }

        if self.check(&Token::Try) {
            self.advance();
            let operand = self.parse_unary();
            return Expr::Try(Box::new(operand), self.span_from(start));
        }

        let expr = self.parse_primary();
        // Postfix record update: `base with Field := Value; … end`
        if self.check(&Token::With) {
            self.parse_record_update(expr, start)
        } else {
            expr
        }
    }
}
