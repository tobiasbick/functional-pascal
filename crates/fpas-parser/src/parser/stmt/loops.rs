use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TO_OR_DOWNTO;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_for_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let (var_name, _) = self.expect_ident().unwrap_or(("_error_".into(), start));
        self.expect(&Token::Colon);
        let var_type = self.parse_type_expr();

        // For-in: `for X: T in Expr do ...`
        if self.eat(&Token::In) {
            let iterable = self.parse_expression();
            self.expect(&Token::Do);
            let body = Box::new(self.parse_statement());
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
            ForDirection::To
        };

        let end_expr = self.parse_expression();
        self.expect(&Token::Do);
        let body = Box::new(self.parse_statement());

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

    pub(super) fn parse_while_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expression();
        self.expect(&Token::Do);
        let body = Box::new(self.parse_statement());
        Stmt::While {
            condition,
            body,
            span: self.span_from(start),
        }
    }

    pub(super) fn parse_repeat_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let body = self.parse_statement_list();
        self.expect(&Token::Until);
        let condition = self.parse_expression();
        Stmt::Repeat {
            body,
            condition,
            span: self.span_from(start),
        }
    }
}
