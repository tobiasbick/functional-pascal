//! Pascal assignment to the enclosing function's name as a way to set its result.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/functions/declarations.md`

use super::super::Checker;
use crate::scope::SymbolKind;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Reports `F := Value` inside function `F` when `F` still names that function, and checks
    /// the value against the result type. Returns whether the assignment was reported.
    pub(crate) fn report_function_name_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
    ) -> bool {
        let [DesignatorPart::Ident(name, _)] = target.parts.as_slice() else {
            return false;
        };
        let Some(function_ctx) = self.scopes.function_ctx.clone() else {
            return false;
        };
        let Some(return_type) = function_ctx.return_type else {
            return false;
        };
        let routine_name = function_ctx
            .name
            .rsplit('.')
            .next()
            .unwrap_or(&function_ctx.name);
        let names_enclosing_function = routine_name.eq_ignore_ascii_case(name)
            && self
                .scopes
                .lookup(name)
                .is_some_and(|symbol| symbol.kind == SymbolKind::Function);
        if !names_enclosing_function {
            return false;
        }
        let value_ty = self.check_expr_with_expected(value, Some(&return_type));
        self.check_type_compat(&return_type, &value_ty, "return value", span);
        let found = format!("{name} := …");
        let habit = ForeignHabit {
            found: &found,
            replacement: "return …",
            explanation: "Assigning to the function name does not set its result; a function returns its value with `return`.",
        };
        let diagnostic = crate::error::sema_error(
            SEMA_IMMUTABLE_ASSIGNMENT,
            format!("Pascal assignment to function name `{name}` does not set its result"),
            String::new(),
            span,
        );
        self.errors.push(habit.attach(diagnostic));
        true
    }
}
