//! `if`, `elsif`, and `while` conditions, including `is` pattern tests.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`

use super::super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_MISPLACED_IS_TEST, SEMA_NON_BOOLEAN_CONDITION,
};
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr, Pattern};

impl Checker {
    /// Checks a branch or loop condition and defines its `is` bindings in the current scope.
    ///
    /// The bindings of an `is` test are visible in later `and` conditions and in the
    /// guarded body; the caller owns that scope.
    pub(in crate::check) fn check_branch_condition(
        &mut self,
        condition: &Expr,
        keyword: &str,
        span: Span,
    ) {
        let mut conjuncts = Vec::new();
        condition.collect_conjuncts(&mut conjuncts);
        if !conjuncts
            .iter()
            .any(|conjunct| matches!(conjunct, Expr::Is { .. }))
        {
            let condition_ty = self.check_expr(condition);
            self.require_boolean_condition(&condition_ty, keyword, span);
            return;
        }
        for conjunct in conjuncts {
            match conjunct {
                Expr::Is { value, pattern, .. } => self.check_is_test(value, pattern),
                other => {
                    let ty = self.check_expr(other);
                    self.require_boolean_condition(&ty, keyword, other.span());
                }
            }
            self.expr_types
                .insert(Self::expr_lookup_key(conjunct), Ty::Boolean);
        }
        record_chain_types(self, condition);
    }

    /// Reports an `is` test outside the condition positions that may bind names.
    pub(in crate::check) fn check_misplaced_is_test(&mut self, expr: &Expr) -> Ty {
        let Expr::Is { value, .. } = expr else {
            return Ty::Error;
        };
        self.error_with_code(
            SEMA_MISPLACED_IS_TEST,
            "`is` pattern tests are only valid as an `if`, `elsif`, or `while` condition",
            "Write `if Value is Pattern then ...`, optionally followed by `and` conditions. `is` cannot appear under `or` or `not`, or as a value; use `case` elsewhere.",
            expr.span(),
        );
        let _ = self.check_expr(value);
        Ty::Boolean
    }

    fn check_is_test(&mut self, value: &Expr, pattern: &Pattern) {
        let value_ty = self.check_expr(value);
        if let Pattern::Value(Expr::Designator(designator)) = pattern
            && designator.parts.len() == 1
            && Self::resolve_designator_name(designator) == "_"
        {
            self.reject_whole_value_wildcard(designator.span);
            return;
        }
        let bindings = self.check_is_pattern(&value_ty, pattern);
        for (name, ty) in bindings {
            self.check_import_alias_collision(&name, pattern.span());
            if !self.define_pattern_binding(&name, &ty, value, pattern.span()) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!(
                        "Pattern binding `{name}` is declared more than once in this condition"
                    ),
                    "Use a distinct binding name in each `is` test of one condition.",
                    pattern.span(),
                );
            }
        }
    }

    fn require_boolean_condition(&mut self, ty: &Ty, keyword: &str, span: Span) {
        if matches!(ty, Ty::GenericParam(..)) {
            self.check_type_compat(&Ty::Boolean, ty, &format!("{keyword} condition"), span);
        } else if !Ty::Boolean.assignment_compatible_with(ty) {
            let (message, hint) = if keyword == "while" {
                (
                    "While condition must be a boolean expression",
                    "while <boolean> do ...",
                )
            } else {
                (
                    "Condition must be a boolean expression",
                    "if <boolean> then ...",
                )
            };
            self.error_with_code(SEMA_NON_BOOLEAN_CONDITION, message, hint, span);
        }
    }
}

fn record_chain_types(checker: &mut Checker, condition: &Expr) {
    if let Expr::BinaryOp {
        op: BinaryOp::And,
        left,
        right,
        ..
    } = condition
    {
        checker
            .expr_types
            .insert(Checker::expr_lookup_key(condition), Ty::Boolean);
        record_chain_types(checker, left);
        record_chain_types(checker, right);
    }
}
