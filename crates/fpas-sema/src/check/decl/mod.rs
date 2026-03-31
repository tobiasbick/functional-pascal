mod consts;
mod routines;
mod types;
mod vars;

use super::Checker;
use crate::types::*;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::*;

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
}
