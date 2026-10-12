//! Statement-local inference from any fully determined initializer.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use fpas_diagnostics::codes::SEMA_TYPE_ANNOTATION_REQUIRED;
use fpas_lexer::Span;
use fpas_parser::Expr;

use super::Checker;
use crate::types::{Ty, needs_expected_type};

impl Checker {
    /// Infers a fixed local type when the initializer supplies complete evidence.
    pub(super) fn infer_local_binding(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
        mutable: bool,
    ) -> Ty {
        let diagnostic_count = self.errors.len();
        let ty = self.check_expr(value);
        if ty.is_error() {
            return ty;
        }
        if self.errors.len() > diagnostic_count {
            return Ty::Error;
        }
        if !needs_expected_type(&ty) {
            return ty;
        }
        self.error_with_code(
            SEMA_TYPE_ANNOTATION_REQUIRED,
            format!("Local binding `{name}` requires a type annotation"),
            format!(
                "Supply a complete type with `{keyword} {name}: Type := ...;`, for example `{keyword} {name}: Option of integer := None;`. An annotation may be omitted when the initializer's type is fully determined.",
                keyword = if mutable { "var" } else { "const" }
            ),
            span,
        );
        Ty::Error
    }
}
