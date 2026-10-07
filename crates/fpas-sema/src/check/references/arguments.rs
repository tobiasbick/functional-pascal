//! Checking of `var Designator` call arguments against their parameters.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::{ParamTy, Ty};
use fpas_diagnostics::codes::{
    SEMA_INVALID_VAR_ARGUMENT, SEMA_TYPE_MISMATCH, SEMA_VAR_ARGUMENT_ALIAS,
    SEMA_VAR_ARGUMENT_MARKER,
};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

/// The variable a `var` argument refers to, used to reject aliasing.
pub(in crate::check) struct VarArgumentRoot {
    /// Scope-qualified identity of the root variable.
    key: String,
    /// Source spelling of the root variable.
    name: String,
    /// Source spelling of the complete argument.
    argument: String,
}

impl Checker {
    /// Checks one argument against a parameter, handling `var` markers on either side.
    ///
    /// Returns the argument type and, for a valid `var` argument, its root variable.
    /// Named calls include the parameter label in marker correction hints.
    pub(in crate::check) fn check_argument_for_param(
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
                (
                    self.check_expr_with_expected_record_literals(other, &param.ty),
                    None,
                )
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
            (false, _) => (
                self.check_expr_with_expected_record_literals(arg, &param.ty),
                None,
            ),
        }
    }

    /// Rejects two `var` arguments of one call that share a root variable.
    pub(in crate::check) fn reject_var_argument_aliases(
        &mut self,
        roots: &[VarArgumentRoot],
        span: Span,
    ) {
        for (index, root) in roots.iter().enumerate() {
            if let Some(earlier) = roots[..index].iter().find(|other| other.key == root.key) {
                self.error_with_code(
                    SEMA_VAR_ARGUMENT_ALIAS,
                    format!(
                        "`var` arguments `{}` and `{}` both refer to `{}`",
                        earlier.argument, root.argument, root.name
                    ),
                    format!(
                        "Pass `{}` once, for example to a routine such as `SwapAt(var {}, I, J)` that changes both parts.",
                        root.name, root.name
                    ),
                    span,
                );
                return;
            }
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

    /// Checks that a `var` argument names a writable variable, record field, or array element.
    fn check_var_argument(&mut self, designator: &Designator) -> (Ty, Option<VarArgumentRoot>) {
        let ty = self.check_designator_expr(designator);
        let Some((symbol, root_parts)) = self.designator_root_symbol(&designator.parts) else {
            return (ty, None);
        };
        let (kind, mutable, root_ty) = (symbol.kind, symbol.mutable, symbol.ty.clone());
        let argument = render_designator(designator);
        let root_name = render_parts(&designator.parts[..root_parts]);
        let writable = mutable && matches!(kind, SymbolKind::Var | SymbolKind::Param);
        if !writable {
            let hint = match kind {
                SymbolKind::Const => format!(
                    "`{root_name}` is a `const` binding; declare it with `var` to pass it as `var {argument}`."
                ),
                SymbolKind::Param => format!(
                    "`{root_name}` is a read-only parameter. Declare it as `var {root_name}` to forward it, or copy it into a local `var` first."
                ),
                SymbolKind::ForVar => {
                    "Loop variables cannot be changed; copy the value into a local `var` first."
                        .to_string()
                }
                _ => "Pass a `var` variable, or a field or element of one.".to_string(),
            };
            self.error_with_code(
                SEMA_INVALID_VAR_ARGUMENT,
                format!("`var {argument}` does not name a writable variable"),
                hint,
                designator.span,
            );
            return (ty, None);
        }
        if !self.var_argument_path_is_storage(&root_ty, &designator.parts[root_parts..]) {
            return (ty, None);
        }
        let key = self.var_root_key(&root_name);
        (
            ty,
            Some(VarArgumentRoot {
                key,
                name: root_name,
                argument,
            }),
        )
    }

    /// Accepts only record fields and array elements below the root variable.
    fn var_argument_path_is_storage(&mut self, root_ty: &Ty, parts: &[DesignatorPart]) -> bool {
        let mut current = self.resolve_visible_type(root_ty);
        for part in parts {
            let next = match (part, &current) {
                (DesignatorPart::Ident(name, _), Ty::Record(record)) => record
                    .fields
                    .iter()
                    .find(|(field, _)| field.eq_ignore_ascii_case(name))
                    .map(|(_, field_ty)| field_ty.clone()),
                (DesignatorPart::Index(_, _), Ty::Array(element)) => Some((**element).clone()),
                _ => None,
            };
            let Some(next) = next else {
                if current.is_error() {
                    return false;
                }
                let (message, span) = match part {
                    DesignatorPart::Ident(name, span) => (
                        format!("`{name}` is not a record field and cannot be passed as `var`"),
                        *span,
                    ),
                    DesignatorPart::Index(_, span) => (
                        format!("An element of `{current}` cannot be passed as `var`"),
                        *span,
                    ),
                };
                self.error_with_code(
                    SEMA_INVALID_VAR_ARGUMENT,
                    message,
                    "A `var` argument is a variable, a record field, or an array element. Copy other values into a local `var`, pass that, and write the result back.",
                    span,
                );
                return false;
            };
            current = self.resolve_visible_type(&next);
        }
        true
    }

    /// Scope-qualified identity of a root variable, merging short and qualified import names.
    fn var_root_key(&self, root_name: &str) -> String {
        let Some((scope, _)) = self.scopes.lookup_with_scope(root_name) else {
            return root_name.to_ascii_lowercase();
        };
        if scope == 0
            && let Some([qualified]) = self
                .imported_candidates
                .get(&root_name.to_ascii_lowercase())
                .map(Vec::as_slice)
        {
            return format!("0:{}", qualified.to_ascii_lowercase());
        }
        format!("{scope}:{}", root_name.to_ascii_lowercase())
    }
}

/// Renders a designator for diagnostics, abbreviating index expressions.
fn render_designator(designator: &Designator) -> String {
    render_parts(&designator.parts)
}

fn render_parts(parts: &[DesignatorPart]) -> String {
    let mut text = String::new();
    for part in parts {
        match part {
            DesignatorPart::Ident(name, _) => {
                if !text.is_empty() {
                    text.push('.');
                }
                text.push_str(name);
            }
            DesignatorPart::Index(index, _) => match index {
                Expr::Designator(inner) if inner.parts.len() == 1 => {
                    text.push('[');
                    text.push_str(&render_parts(&inner.parts));
                    text.push(']');
                }
                Expr::Integer(value, _) => text.push_str(&format!("[{value}]")),
                _ => text.push_str("[…]"),
            },
        }
    }
    text
}
