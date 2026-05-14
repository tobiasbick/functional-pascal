use super::super::super::super::Compiler;
use fpas_parser::{CaseLabel, DesignatorPart, Expr};

impl Compiler {
    pub(super) fn scalar_guard_binding_name(&self, label: &CaseLabel) -> Option<String> {
        let CaseLabel::Value {
            start, end: None, ..
        } = label
        else {
            return None;
        };
        if !self.is_scalar_guard_binding_expr(start) {
            return None;
        }

        let Expr::Designator(designator) = start else {
            return None;
        };
        let DesignatorPart::Ident(name, _) = &designator.parts[0] else {
            return None;
        };
        Some(name.clone())
    }

    pub(super) fn is_scalar_guard_binding_expr(&self, expr: &Expr) -> bool {
        self.scalar_case_bindings
            .contains(&fpas_sema::expr_lookup_key(expr))
    }
}
