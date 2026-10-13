use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
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
