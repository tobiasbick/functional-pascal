use super::super::Parser;
use crate::ast::*;

impl Parser {
    /// Parse `go <call-expression>`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    pub(crate) fn parse_go_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance(); // consume `go`
        let expr = self.parse_primary();
        Stmt::Go {
            expr,
            span: self.span_from(start),
        }
    }
}
