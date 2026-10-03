//! Operator precedence from `docs/pascal/language/basics/operators.md`.

use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Parses same-operator boolean chains and rejects ungrouped mixtures.
    pub(super) fn parse_logical(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_not();
        let mut first = None;
        loop {
            let op = match self.current_token() {
                Token::And => BinaryOp::And,
                Token::Or => BinaryOp::Or,
                Token::Xor => BinaryOp::Xor,
                _ => break,
            };
            if first.is_some_and(|previous| previous != op) {
                self.error_with_code(PARSE_EXPECTED_EXPRESSION,
                    "Mixed logical operators require parentheses",
                    "Group the intended operations, for example `(A and B) or C` or `A and (B or C)`.",
                    self.current_span());
            }
            first.get_or_insert(op);
            self.advance();
            let right = self.parse_not();
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            };
        }
        left
    }

    fn parse_not(&mut self) -> Expr {
        if self.check(&Token::Not) {
            self.with_nesting(Self::parse_not_inner)
        } else {
            self.parse_comparison()
        }
    }

    fn parse_not_inner(&mut self) -> Expr {
        let start = self.advance().span;
        let operand = self.parse_not();
        Expr::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span: self.span_from(start),
        }
    }

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
            "Use two comparisons, for example `(A < B) and (B < C)`. If the middle expression has effects, evaluate it once into a local binding and compare that binding twice.",
            span,
        );
        while self.is_comparison_token() {
            self.advance();
            let _ = self.parse_additive();
        }
    }

    /// Parses left-associative addition and subtraction.
    pub(super) fn parse_additive(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_multiplicative();

        loop {
            let op = match self.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
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

    /// Parses arithmetic products and diagnoses obsolete infix shifts.
    pub(super) fn parse_multiplicative(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_unary();

        loop {
            let op = match self.current_token() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::RealDiv,
                Token::Div => BinaryOp::IntDiv,
                Token::Mod => BinaryOp::Mod,
                Token::Ident(name)
                    if name.eq_ignore_ascii_case("shl") || name.eq_ignore_ascii_case("shr") =>
                {
                    self.error_with_code(PARSE_EXPECTED_EXPRESSION,
                        "Shift operators have been replaced by Std.Bits functions",
                        "Import `uses Std.Bits as Bits;` and use `Bits.ShiftLeft(Value, Count)` or `Bits.ShiftRight(Value, Count)`.", self.current_span());
                    self.advance();
                    let _ = self.parse_unary();
                    continue;
                }
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

    /// Parses numeric negation and error propagation above multiplication.
    pub(super) fn parse_unary(&mut self) -> Expr {
        self.with_nesting(Self::parse_unary_inner)
    }

    fn parse_unary_inner(&mut self) -> Expr {
        let start = self.current_span();

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
