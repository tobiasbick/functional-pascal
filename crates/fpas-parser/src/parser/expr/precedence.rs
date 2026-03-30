use super::super::Parser;
use crate::ast::*;
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
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let right = self.parse_additive();
            Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            }
        } else {
            left
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
