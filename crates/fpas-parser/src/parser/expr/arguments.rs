//! Call argument lists with positional or named arguments.
//!
//! **Documentation:** `docs/pascal/language/functions/parameters.md`

use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_MIXED_CALL_ARGUMENTS};
use fpas_lexer::Token;

impl Parser {
    /// Parses comma-separated arguments, rejecting expression-owned final terminators.
    ///
    /// A call is fully positional or fully named; a mixed list is reported and
    /// recovered as positional arguments.
    pub(crate) fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        loop {
            let arg = self.parse_argument();
            if matches!(
                arg.argument_value(),
                Expr::Closure(_) | Expr::RecordUpdate { .. }
            ) && self.check(&Token::Semicolon)
                && matches!(self.peek_token(), Token::RParen | Token::Comma)
            {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Expression arguments do not have a terminating `;`",
                    "Remove `;` before the argument separator or `)`, for example `Apply(function(): integer begin return 1; end function)`.",
                    self.current_span(),
                );
                self.advance();
            }
            args.push(arg);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.reject_mixed_arguments(args)
    }

    /// Parses one positional or named argument, including `Name := var Designator`.
    pub(super) fn parse_argument(&mut self) -> Expr {
        if !(matches!(self.current_token(), Token::Ident(_))
            && matches!(self.peek_token(), Token::ColonAssign))
        {
            return self.parse_argument_value();
        }
        let start = self.current_span();
        let (name, name_span) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        self.expect(&Token::ColonAssign);
        let value = self.parse_argument_value();
        Expr::NamedArgument {
            name,
            name_span,
            value: Box::new(value),
            span: self.span_from(start),
        }
    }

    fn parse_argument_value(&mut self) -> Expr {
        if self.check(&Token::Var) {
            let start = self.advance().span;
            let designator = self.parse_designator();
            Expr::VarArgument {
                designator,
                span: self.span_from(start),
            }
        } else {
            self.parse_expression()
        }
    }

    fn reject_mixed_arguments(&mut self, args: Vec<Expr>) -> Vec<Expr> {
        let first_named = args
            .first()
            .is_some_and(|arg| arg.argument_name().is_some());
        let Some(mismatch) = args
            .iter()
            .find(|arg| arg.argument_name().is_some() != first_named)
        else {
            return args;
        };
        self.error_with_code(
            PARSE_MIXED_CALL_ARGUMENTS,
            "A call cannot mix positional and named arguments",
            "Pass every argument by position, for example `Move(1, 2)`, or every argument by name, for example `Move(Dx := 1, Dy := 2)`.",
            mismatch.span(),
        );
        args.into_iter()
            .map(|arg| match arg {
                Expr::NamedArgument { value, .. } => *value,
                positional => positional,
            })
            .collect()
    }
}
