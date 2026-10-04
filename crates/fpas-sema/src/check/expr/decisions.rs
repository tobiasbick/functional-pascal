//! Expected and order-independent common types for selected value branches.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{CaseArm, CaseExpr, Expr, IfExpr};

impl Checker {
    /// Check conditions and infer one order-independent type for the selected values.
    pub(super) fn check_if_expression(&mut self, decision: &IfExpr, expected: Option<&Ty>) -> Ty {
        self.check_if_condition(&decision.condition, decision.condition.span());
        for (condition, _) in &decision.elsif_values {
            self.check_if_condition(condition, condition.span());
        }
        let values = std::iter::once(&decision.then_value)
            .chain(decision.elsif_values.iter().map(|(_, value)| value))
            .chain(std::iter::once(&decision.else_value))
            .collect::<Vec<_>>();
        self.inference_depth += 1;
        let types = values
            .iter()
            .map(|value| self.check_decision_value(value, expected))
            .collect::<Vec<_>>();
        self.inference_depth -= 1;
        let result = self.decision_type(&types, expected, decision.span);
        if !result.is_error() {
            for (value, ty) in values.iter().zip(&types) {
                if ty.has_inference_holes() {
                    self.check_expr_with_expected(value, &result);
                }
            }
        }
        result
    }

    /// Check scoped pattern bindings, exhaustive coverage, and a common branch type.
    pub(super) fn check_case_expression(
        &mut self,
        decision: &CaseExpr,
        expected: Option<&Ty>,
    ) -> Ty {
        let matched = self.check_expr(&decision.value);
        self.check_case_expression_type(&matched, decision.span);
        let mut types =
            Vec::with_capacity(decision.arms.len() + usize::from(decision.else_value.is_some()));
        let mut bindings = Vec::with_capacity(decision.arms.len());
        self.inference_depth += 1;
        for arm in &decision.arms {
            let binding_sets = arm
                .labels
                .iter()
                .map(|pattern| self.check_pattern(pattern, &matched))
                .collect();
            let names = self.shared_case_arm_bindings(binding_sets, arm.span);
            self.push_value_arm_scope(
                arm,
                &names,
                self.expr_is_task_bound(Self::expr_lookup_key(&decision.value)),
            );
            self.check_guard(&arm.guard, arm.span);
            types.push(self.check_decision_value(&arm.body, expected));
            self.scopes.pop_scope();
            bindings.push(names);
        }
        if let Some(value) = &decision.else_value {
            types.push(self.check_decision_value(value, expected));
        }
        self.inference_depth -= 1;
        self.check_recursive_case_coverage(
            &matched,
            &decision.arms,
            decision.else_value.is_some(),
            true,
            decision.span,
        );
        let result = self.decision_type(&types, expected, decision.span);
        if !result.is_error() {
            for ((arm, ty), names) in decision.arms.iter().zip(&types).zip(&bindings) {
                if ty.has_inference_holes() {
                    self.push_value_arm_scope(
                        arm,
                        names,
                        self.expr_is_task_bound(Self::expr_lookup_key(&decision.value)),
                    );
                    self.check_expr_with_expected(&arm.body, &result);
                    self.scopes.pop_scope();
                }
            }
            if let Some(value) = &decision.else_value
                && types.last().is_some_and(Ty::has_inference_holes)
            {
                self.check_expr_with_expected(value, &result);
            }
        }
        result
    }

    fn push_value_arm_scope(
        &mut self,
        arm: &CaseArm<Expr>,
        bindings: &[(String, Ty)],
        task_bound: bool,
    ) {
        self.scopes.push_scope();
        for (name, ty) in bindings {
            let declaration = arm
                .labels
                .first()
                .and_then(|pattern| pattern.binding(name).map(|binding| binding.span()))
                .unwrap_or(arm.span);
            self.scopes.define_with_declaration(
                name,
                Symbol {
                    ty: ty.clone(),
                    kind: SymbolKind::Var,
                    mutable: false,
                    task_bound: task_bound && self.type_can_contain_callable(ty),
                },
                declaration,
            );
        }
    }

    fn check_decision_value(&mut self, value: &Expr, expected: Option<&Ty>) -> Ty {
        let ty = match expected {
            Some(expected) => self.check_expr_with_expected(value, expected),
            None => self.check_expr(value),
        };
        if matches!(ty, Ty::Unit) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "A decision expression branch must produce a value",
                "Use a function or ordinary value; a procedure call produces no value.",
                value.span(),
            );
            return Ty::Error;
        }
        ty
    }

    fn decision_type(&mut self, types: &[Ty], expected: Option<&Ty>, span: Span) -> Ty {
        if types.is_empty() || types.iter().any(Ty::is_error) {
            return Ty::Error;
        }
        let mut result = expected.cloned().unwrap_or_else(|| types[0].clone());
        for actual in types {
            if expected.is_none()
                && actual.assignment_compatible_with(&result)
                && !result.assignment_compatible_with(actual)
            {
                result = actual.clone();
            }
            if !result.assignment_compatible_with(actual) {
                self.error_with_code(SEMA_TYPE_MISMATCH, format!("Decision branches have incompatible types `{result}` and `{actual}`"),
                    "Make every branch return the same ordinary type, or supply an explicit compatible annotation.", span);
                return Ty::Error;
            }
            result = result.complete_inference_with(actual);
        }
        if result.has_inference_holes() && self.inference_depth == 0 {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Cannot infer the complete type of this decision expression",
                "Add a type annotation for empty collections or payloadless generic constructors.",
                span,
            );
            return Ty::Error;
        }
        result
    }
}
