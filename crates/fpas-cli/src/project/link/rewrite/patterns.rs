use super::NameRewriter;
use fpas_parser::{CaseArm, CaseLabel, Designator, DesignatorPart, Expr};
use std::collections::HashSet;

impl NameRewriter<'_> {
    pub(super) fn case_arm_bindings(arm: &CaseArm) -> Vec<String> {
        let mut bindings = Vec::new();
        let mut seen = HashSet::new();
        for label in &arm.labels {
            Self::collect_case_label_bindings(label, &mut bindings, &mut seen);
        }
        bindings
    }

    fn collect_case_label_bindings(
        label: &CaseLabel,
        bindings: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        match label {
            CaseLabel::Value { start, end, .. } => {
                if end.is_none() {
                    Self::collect_pattern_bindings_from_expr(start, false, bindings, seen);
                }
            }
            CaseLabel::Destructure { binding, .. } => {
                if let Some(name) = binding {
                    Self::push_binding_name(name, bindings, seen);
                }
            }
        }
    }

    fn collect_pattern_bindings_from_expr(
        expr: &Expr,
        allow_binding_name: bool,
        bindings: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Call { args, .. } => {
                for arg in args {
                    Self::collect_pattern_bindings_from_expr(arg, true, bindings, seen);
                }
            }
            Expr::Designator(designator) if allow_binding_name => {
                if let Some(name) = Self::pattern_binding_name(designator) {
                    Self::push_binding_name(name, bindings, seen);
                }
            }
            Expr::Paren(inner, _) => {
                Self::collect_pattern_bindings_from_expr(inner, allow_binding_name, bindings, seen);
            }
            _ => {}
        }
    }

    fn push_binding_name(name: &str, bindings: &mut Vec<String>, seen: &mut HashSet<String>) {
        if name != "_" && seen.insert(name.to_string()) {
            bindings.push(name.to_string());
        }
    }

    pub(super) fn is_pattern_binding_designator(designator: &Designator) -> bool {
        Self::pattern_binding_name(designator).is_some()
    }

    fn pattern_binding_name(designator: &Designator) -> Option<&str> {
        if designator.parts.len() != 1 {
            return None;
        }
        match &designator.parts[0] {
            DesignatorPart::Ident(name, _) if name != "_" => Some(name.as_str()),
            _ => None,
        }
    }
}
