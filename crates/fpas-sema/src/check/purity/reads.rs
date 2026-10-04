//! Pure reads and closure creation preserve lexical storage ownership.

use crate::check::{CaptureBinding, Checker};
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::DesignatorPart;

impl Checker {
    pub(in crate::check) fn check_pure_read(&mut self, name: &str, span: Span) {
        let Some(context) = self.pure_evaluation else {
            return;
        };
        let Some((scope_index, symbol)) = self.scopes.lookup_with_scope(name) else {
            return;
        };
        if matches!(
            symbol.kind,
            SymbolKind::Type | SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
        ) {
            return;
        }
        // A named generic pure function is checked at its concrete call/instantiation.
        let pure_routine = symbol.kind == SymbolKind::Function
            && matches!(&symbol.ty, Ty::Function(function) if function.pure);
        let allowed_type = pure_routine || self.is_pure_data(&symbol.ty);
        if !allowed_type || (scope_index < context.scope_index && symbol.mutable) {
            self.error_with_code(SEMA_TYPE_MISMATCH,
                format!("Pure evaluation cannot read `{name}`"),
                "Read immutable resource-free data or pure callables; mutable data must belong to the current function.", span);
        }
    }

    pub(in crate::check) fn check_pure_designator(&mut self, parts: &[DesignatorPart], span: Span) {
        if self.pure_evaluation.is_none() {
            return;
        }
        let mut prefix = String::new();
        for part in parts {
            let DesignatorPart::Ident(name, _) = part else {
                break;
            };
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(name);
            let resolved = self.qualified_import_name(&prefix);
            if self.scopes.lookup(&resolved).is_some() {
                self.check_pure_read(&resolved, span);
                break;
            }
        }
    }

    pub(in crate::check) fn check_pure_captures(
        &mut self,
        captures: &[CaptureBinding],
        span: Span,
    ) {
        for capture in captures {
            if capture.mutable || capture.task_bound || !self.is_pure_data(&capture.ty) {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Pure evaluation cannot capture `{}`", capture.name),
                    "Capture only immutable resource-free data or pure callable values.",
                    span,
                );
            }
        }
    }
}
