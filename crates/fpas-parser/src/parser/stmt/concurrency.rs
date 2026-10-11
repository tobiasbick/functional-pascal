use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    /// Parse `go <call-expression>`.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    pub(crate) fn parse_go_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance(); // consume `go`
        let expr = self.parse_go_call_expression(start);
        Stmt::Go {
            expr,
            span: self.span_from(start),
        }
    }

    /// Parse `scope <statements> end scope`.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`
    pub(super) fn parse_scope_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::Scope, |parser| {
            let start = parser.current_span();
            parser.advance(); // consume `scope`
            let body = parser.parse_statement_list();
            parser.require_control_statements(&body);
            parser.expect_block_end(&Token::Scope);
            Stmt::Scope {
                body,
                span: parser.span_from(start),
            }
        })
    }
}
