//! Record routine lookup.

use super::Checker;
use crate::types::{MethodKind, Ty};

impl Checker {
    /// Resolve a declared record routine by its qualified name.
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

    /// Resolve a declared record routine by its qualified name.
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
        if let Some(symbol) = self.scopes.lookup(&record_ty.name)
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

    /// Resolve a declared record routine by its qualified name.
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

        if let Some(symbol) = self.scopes.lookup(&record_ty.name)
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

    /// Resolve a declared record routine by its qualified name.
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
}
