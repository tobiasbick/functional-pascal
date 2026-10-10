//! Distinct conversion labels such as `when UserId(1):` in `case` and `is` patterns.
//!
//! **Documentation:** `docs/pascal/language/types/distinct-types.md`

use super::Checker;
use super::coverage::Pat;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, Pattern};

impl Checker {
    /// Checks `Name(Value)` as a constant label when `Name` is a distinct type.
    ///
    /// Returns `None` when `Name` is not a distinct type, so the caller keeps its
    /// variant or call handling.
    pub(super) fn check_distinct_label(
        &mut self,
        expected_ty: &Ty,
        constructor: &Designator,
        pattern: &Pattern,
        span: Span,
    ) -> Option<Pat> {
        let name = Self::resolve_designator_name(constructor);
        let symbol = self.scopes.lookup(&name)?;
        if symbol.kind != SymbolKind::Type {
            return None;
        }
        let Ty::Distinct(label) = self.resolve_visible_type(&symbol.ty.clone()) else {
            return None;
        };
        let Some(argument) = pattern.conversion_argument() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Case label `{name}(...)` takes exactly one constant value"),
                format!("Write the label as a conversion, for example `when {name}(1):`."),
                span,
            );
            return Some(Pat::Wild);
        };
        let errors_before = self.errors.len();
        self.check_type_compat(
            expected_ty,
            &Ty::Distinct(label.clone()),
            "case label",
            span,
        );
        let argument_ty = self.check_expr(argument);
        self.check_type_compat(
            &label.underlying,
            &argument_ty,
            "case label value",
            argument.span(),
        );
        if self.errors.len() != errors_before {
            return Some(Pat::Wild);
        }
        self.require_case_constant(argument);
        Some(self.value_constructor(&label.underlying, argument))
    }
}

/// Returns the underlying type a distinct `case` selector or pattern value compares by.
pub(super) fn comparison_type(ty: &Ty) -> &Ty {
    match ty {
        Ty::Distinct(distinct) => &distinct.underlying,
        other => other,
    }
}
