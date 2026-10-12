//! Visible instance/static record routine lookup.
//! Documentation: docs/pascal/language/types/record-methods.md.

use crate::check::Checker;
use crate::scope::SymbolKind;
use crate::types::{MethodKind, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    /// Resolves a visible instance method signature.
    pub(in crate::check) fn resolve_method_kind(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
        qualified: &str,
    ) -> Option<MethodKind> {
        if !self.record_member_is_visible(record_ty, method_name) {
            return None;
        }
        if let Some((_, method_kind)) = record_ty
            .methods
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(method_kind.clone());
        }

        let symbol = self.scopes.lookup(qualified)?;
        match &symbol.ty {
            Ty::Function(function_ty) => Some(MethodKind::Function(function_ty.clone())),
            Ty::Procedure(procedure_ty) => Some(MethodKind::Procedure(procedure_ty.clone())),
            _ => None,
        }
    }

    /// Resolves a visible static function signature.
    pub(in crate::check) fn resolve_static_function(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
        _qualified: &str,
    ) -> Option<crate::types::FunctionTy> {
        if !self.record_member_is_visible(record_ty, method_name) {
            return None;
        }
        if let Some((_, function_ty)) = record_ty
            .static_functions
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(function_ty.clone());
        }

        // RecordTy clones on values may omit the table; consult the type symbol.
        if let Some(symbol) = self.scopes.lookup_type(&record_ty.name)
            && let Ty::Record(stored) = &symbol.ty
            && let Some((_, function_ty)) = stored
                .static_functions
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(function_ty.clone());
        }
        None
    }

    /// Resolves a visible static procedure signature.
    pub(in crate::check) fn resolve_static_procedure(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
    ) -> Option<crate::types::ProcedureTy> {
        if !self.record_member_is_visible(record_ty, method_name) {
            return None;
        }
        if let Some((_, procedure_ty)) = record_ty
            .static_procedures
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(procedure_ty.clone());
        }

        if let Some(symbol) = self.scopes.lookup_type(&record_ty.name)
            && let Ty::Record(stored) = &symbol.ty
            && let Some((_, procedure_ty)) = stored
                .static_procedures
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(procedure_ty.clone());
        }
        None
    }

    /// Identifies static routine members before checking a value receiver.
    pub(in crate::check) fn static_routine_kind_on_record(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
    ) -> Option<&'static str> {
        if self
            .resolve_static_function(record_ty, method_name, "")
            .is_some()
        {
            Some("function")
        } else if self
            .resolve_static_procedure(record_ty, method_name)
            .is_some()
        {
            Some("procedure")
        } else {
            None
        }
    }

    /// True when the designator names a type symbol (possibly qualified).
    pub(in crate::check) fn designator_denotes_type(&self, designator: &Designator) -> bool {
        let name = Self::resolve_designator_name(designator);
        self.scopes
            .lookup(&name)
            .is_some_and(|symbol| matches!(symbol.kind, SymbolKind::Type))
    }

    /// Reject `Type.InstanceMethod(...)` looked up as a free function through the type name.
    pub(in crate::check) fn reject_instance_method_through_type(
        &mut self,
        designator: &Designator,
        span: Span,
    ) -> bool {
        if designator.parts.len() < 2 {
            return false;
        }
        let method_name = match designator.parts.last() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => return false,
        };
        let receiver = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };
        if !self.designator_denotes_type(&receiver) {
            return false;
        }
        let receiver_ty = self.check_designator_expr(&receiver);
        let Ty::Record(record_ty) = self.resolve_visible_type(&receiver_ty) else {
            return false;
        };
        let is_instance = record_ty
            .methods
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(&method_name));
        if !is_instance {
            return false;
        }
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{method_name}` is an instance method and must be called through a `{}.…` value",
                record_ty.name
            ),
            format!(
                "Call it as `Value.{method_name}(...)` where `Value` has type `{}`.",
                record_ty.name
            ),
            span,
        );
        true
    }
}
