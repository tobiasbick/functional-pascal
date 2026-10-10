mod consts;
mod routines;
/// Type declarations, whole-unit collection, and record member checking.
pub(crate) mod types;
mod vars;

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::*;
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH};
use fpas_parser::*;
use std::collections::HashSet;

impl Checker {
    pub(crate) fn check_decl(&mut self, decl: &Decl) {
        let (name, span) = match decl {
            Decl::Const(d) => (&d.name, d.span),
            Decl::Var(d) => (&d.name, d.span),
            Decl::TypeDef(d) => (&d.name, d.span),
            Decl::Function(d) => (&d.name, d.span),
            Decl::Procedure(d) => (&d.name, d.span),
        };
        if self.check_import_alias_collision(name, span) {
            return;
        }
        match decl {
            Decl::Const(c) => self.check_const_def(c),
            Decl::Var(v) => self.check_var_def(v, true),
            Decl::TypeDef(td) => self.check_type_def(td),
            Decl::Function(f) => self.check_function_decl(f),
            Decl::Procedure(p) => self.check_procedure_decl(p),
        }
    }

    /// Compare canonical types, including recursive nominal references.
    pub(crate) fn check_type_compat(
        &mut self,
        expected: &Ty,
        actual: &Ty,
        context: &str,
        span: fpas_lexer::Span,
    ) {
        let expected = self.resolve_visible_type(expected);
        let actual = self.resolve_visible_type(actual);
        if !expected.assignment_compatible_with(&actual) {
            let hint = Self::distinct_conversion_hint(&expected, &actual)
                .unwrap_or_else(|| format!("The {context} must match the declared type."));
            self.errors.push(
                crate::error::sema_error(
                    SEMA_TYPE_MISMATCH,
                    format!("Type mismatch in {context}: expected `{expected}`, found `{actual}`"),
                    hint,
                    span,
                )
                .with_expected_found(expected.to_string(), actual.to_string()),
            );
        }
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
            self.check_import_alias_collision(&type_param.name, span);
            if !seen.insert(canonical_symbol_name(&type_param.name)) {
                self.report_duplicate_declaration("type parameter", &type_param.name, span);
            }
        }
    }

    pub(crate) fn check_unique_formal_param_names(&mut self, params: &[FormalParam]) {
        let mut seen = HashSet::new();
        for param in params {
            self.check_import_alias_collision(&param.name, param.span);
            if !seen.insert(canonical_symbol_name(&param.name)) {
                self.report_duplicate_declaration("parameter", &param.name, param.span);
            }
        }
    }
}
