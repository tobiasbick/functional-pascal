//! Checking of `var Designator` call arguments against their parameters.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use super::super::Checker;
use super::storage::{VarArgumentRoot, render_designator};
use crate::types::{ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_VAR_ARGUMENT_MARKER};
use fpas_parser::{Designator, Expr};

impl Checker {
    /// Checks one argument against a parameter, handling `var` markers on either side.
    ///
    /// Returns the argument type and, for a valid `var` argument, its root variable.
    /// Named calls include the parameter label in marker correction hints.
    pub(crate) fn check_argument_for_param(
        &mut self,
        callee: &str,
        param: &ParamTy,
        arg: &Expr,
        named: bool,
    ) -> (Ty, Option<VarArgumentRoot>) {
        let label = if named {
            format!("{} := ", param.name)
        } else {
            String::new()
        };
        match (param.is_var(), arg) {
            (true, Expr::VarArgument { designator, .. }) => {
                let (ty, root) = self.check_var_argument(designator);
                if !ty.is_error()
                    && !param.ty.is_error()
                    && !(param.ty.compatible_with(&ty) && ty.compatible_with(&param.ty))
                {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "`var` argument `{}` has type `{ty}`, but parameter `{}` of `{callee}` has type `{}`",
                            render_designator(designator),
                            param.name,
                            param.ty
                        ),
                        "A `var` argument must have exactly the parameter type because the routine writes it back.",
                        arg.span(),
                    );
                    return (Ty::Error, root);
                }
                (ty, root)
            }
            (true, other) => {
                let hint = match other {
                    Expr::Designator(designator) => format!(
                        "Write `{label}var {}` so the change to the caller's variable is visible at the call site.",
                        render_designator(designator)
                    ),
                    _ if param.ty.is_error() => format!(
                        "Declare a writable variable and pass `{label}var Temp`; a computed value cannot be passed as `var`."
                    ),
                    _ => format!(
                        "A `var` parameter needs a variable. Declare one, for example `var Temp: {} := …;`, and pass `{label}var Temp`.",
                        param.ty
                    ),
                };
                self.error_with_code(
                    SEMA_VAR_ARGUMENT_MARKER,
                    format!(
                        "Parameter `{}` of `{callee}` is a `var` parameter; its argument must be marked with `var`",
                        param.name
                    ),
                    hint,
                    other.span(),
                );
                (self.check_expr(other), None)
            }
            (false, Expr::VarArgument { designator, .. }) => {
                self.error_with_code(
                    SEMA_VAR_ARGUMENT_MARKER,
                    format!(
                        "Parameter `{}` of `{callee}` is read-only; `var` is only valid for `var` parameters",
                        param.name
                    ),
                    format!(
                        "Remove `var` and pass `{label}{}`.",
                        render_designator(designator)
                    ),
                    arg.span(),
                );
                (self.check_designator_expr(designator), None)
            }
            (false, _) => (self.check_expr(arg), None),
        }
    }

    /// Reports a `var` argument outside a call that accepts it.
    pub(crate) fn check_misplaced_var_argument(&mut self, expr: &Expr) -> Ty {
        let Expr::VarArgument { designator, span } = expr else {
            return Ty::Error;
        };
        self.error_with_code(
            SEMA_VAR_ARGUMENT_MARKER,
            format!(
                "`var {}` is only valid as the argument of a `var` parameter",
                render_designator(designator)
            ),
            "Remove `var` to pass the value, or call a routine that declares a `var` parameter.",
            *span,
        );
        let _ = self.check_designator_expr(designator);
        Ty::Error
    }

    fn check_var_argument(&mut self, designator: &Designator) -> (Ty, Option<VarArgumentRoot>) {
        let ty = self.check_designator_expr(designator);
        self.check_var_storage(designator, ty)
    }
}
