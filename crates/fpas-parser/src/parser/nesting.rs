//! Shared parser recursion budget.
//!
//! **Documentation:** `docs/pascal/program-structure/cli.md` (Checking without running).

use super::Parser;
use fpas_diagnostics::codes::PARSE_NESTING_LIMIT_EXCEEDED;

pub(crate) const MAX_PARSER_NESTING_DEPTH: usize = 128;

impl Parser {
    pub(super) fn with_nesting<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        if self.nesting_depth >= MAX_PARSER_NESTING_DEPTH && !self.nesting_limit_reached {
            let span = self.current_span();
            self.error_with_code(
                PARSE_NESTING_LIMIT_EXCEEDED,
                "Parser nesting limit exceeded",
                &format!(
                    "Reduce nested expressions, statements, types, or routine declarations to at most {MAX_PARSER_NESTING_DEPTH} levels."
                ),
                span,
            );
            self.nesting_limit_reached = true;
            self.pos = self.tokens.len() - 1;
        }

        self.nesting_depth += 1;
        let result = parse(self);
        self.nesting_depth -= 1;
        result
    }
}
