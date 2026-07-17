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
        if !expected.compatible_with(actual) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in {context}: expected `{expected}`, found `{actual}`"),
                format!("The {context} must match the declared type."),
                span,
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
