mod consts;
mod routines;
mod types;
mod vars;

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::*;
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH};
use fpas_parser::*;
use std::collections::HashSet;

impl Checker {
    pub(crate) fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Const(c) => self.check_const_def(c),
            Decl::Var(v) => self.check_var_def(v, false),
            Decl::MutableVar(v) => self.check_var_def(v, true),
            Decl::TypeDef(td) => self.check_type_def(td),
            Decl::Function(f) => self.check_function_decl(f),
            Decl::Procedure(p) => self.check_procedure_decl(p),
        }
    }

    pub(crate) fn check_type_compat(
        &mut self,
        expected: &Ty,
        actual: &Ty,
        context: &str,
        span: fpas_lexer::Span,
    ) {
        if !expected.assignment_compatible_with(actual)
            && !self.private_records_are_compatible_inside_owner(expected, actual)
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in {context}: expected `{expected}`, found `{actual}`"),
                format!("The {context} must match the declared type."),
                span,
            );
        }
    }

    fn private_records_are_compatible_inside_owner(&self, expected: &Ty, actual: &Ty) -> bool {
        let (Ty::Record(expected), Ty::Record(actual)) = (expected, actual) else {
            return false;
        };
        if expected.private_members.is_empty() && actual.private_members.is_empty() {
            return false;
        }
        let current_owner = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.as_deref());
        let private_records_are_owned_here = [expected.as_ref(), actual.as_ref()]
            .into_iter()
            .filter(|record| !record.private_members.is_empty())
            .all(|record| record.owner_unit.as_deref() == current_owner);

        private_records_are_owned_here
            && Ty::record_fields_assignment_compatible(&expected.fields, &actual.fields)
    }

    fn report_duplicate_declaration(&mut self, kind: &str, name: &str, span: fpas_lexer::Span) {
        self.error_with_code(
            SEMA_DUPLICATE_DECLARATION,
            format!("Duplicate {kind} `{name}`"),
            format!("Each {kind} name must be unique in the same scope."),
            span,
        );
    }

    pub(super) fn check_unique_type_param_names(
        &mut self,
        type_params: &[TypeParam],
        span: fpas_lexer::Span,
    ) {
        let mut seen = HashSet::new();
        for type_param in type_params {
            if !seen.insert(canonical_symbol_name(&type_param.name)) {
                self.report_duplicate_declaration("type parameter", &type_param.name, span);
            }
        }
    }

    pub(crate) fn check_unique_formal_param_names(&mut self, params: &[FormalParam]) {
        let mut seen = HashSet::new();
        for param in params {
            if !seen.insert(canonical_symbol_name(&param.name)) {
                self.report_duplicate_declaration("parameter", &param.name, param.span);
            }
        }
    }
}
