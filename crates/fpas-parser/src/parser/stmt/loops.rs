use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TO_OR_DOWNTO;
use fpas_lexer::Token;

impl Parser {
    /// Parses counting and collection loops with scoped bodies and `end for`.
    ///
    /// **Documentation:** `docs/pascal/language/control-flow/for-loops.md`
    pub(super) fn parse_for_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::For, Self::parse_for_stmt_inner)
    }

    fn parse_for_stmt_inner(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let (var_name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        self.expect(&Token::Colon);
        let var_type = self.parse_type_expr();

        // For-in: `for X: T in Expr do ...`
        if self.eat(&Token::In) {
            let iterable = self.parse_expression();
            let body_start = self.expect(&Token::Do).unwrap_or(self.current_span());
            let body = Box::new(self.parse_control_body(body_start));
            self.expect_block_end(&Token::For);
            return Stmt::ForIn {
                var_name,
                var_type,
                iterable,
                body,
                span: self.span_from(start),
            };
        }

        // Classic for: `for X: T := Start to/downto End do ...`
        self.expect(&Token::ColonAssign);
        let start_expr = self.parse_expression();

        let direction = if self.eat(&Token::To) {
            ForDirection::To
        } else if self.eat(&Token::Downto) {
            ForDirection::Downto
        } else {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TO_OR_DOWNTO,
                "Expected `to` or `downto` in for loop",
                "for I: integer := 0 to 10 do ...",
                span,
            );
            // Recovery default so the rest of the loop still parses.
            ForDirection::To
        };

        let end_expr = self.parse_expression();
        let body_start = self.expect(&Token::Do).unwrap_or(self.current_span());
        let body = Box::new(self.parse_control_body(body_start));
        self.expect_block_end(&Token::For);

        Stmt::For {
            var_name,
            var_type,
            start: start_expr,
            direction,
            end: end_expr,
            body,
            span: self.span_from(start),
        }
    }

    /// Parses a while loop with a scoped body and `end while`.
    ///
    /// **Documentation:** `docs/pascal/language/control-flow/while-repeat.md`
    pub(super) fn parse_while_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::While, Self::parse_while_stmt_inner)
    }

    fn parse_while_stmt_inner(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expression();
        let body_start = self.expect(&Token::Do).unwrap_or(self.current_span());
        let body = Box::new(self.parse_control_body(body_start));
        self.expect_block_end(&Token::While);
        Stmt::While {
            condition,
            body,
            span: self.span_from(start),
        }
    }

    /// Parses a terminated statement list before the repeat loop's condition.
    ///
    /// **Documentation:** `docs/pascal/language/control-flow/while-repeat.md`
    pub(super) fn parse_repeat_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let body = self.parse_statement_list();
        self.require_control_statements(&body);
        self.expect(&Token::Until);
        let condition = self.parse_expression();
        Stmt::Repeat {
            body,
            condition,
            span: self.span_from(start),
        }
    }
}
