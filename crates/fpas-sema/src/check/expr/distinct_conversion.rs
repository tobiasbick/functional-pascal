//! Explicit conversions into and out of distinct domain types.
//!
//! `UserId(42)` wraps an underlying value and `integer(Id)` unwraps it. Both are
//! identity conversions at runtime; implicit conversion in either direction is a
//! type mismatch.
//!
//! **Documentation:** `docs/pascal/language/types/distinct-types.md`

use super::Checker;
use crate::types::{DistinctTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;

const SCALAR_CONVERSION_HINT: &str = "Convert between built-in types with conversion routines such as `IntToReal`, `Trunc`, `Round`, `IntToStr`, `StrToInt`, or `BoolToStr`.";

impl Checker {
    /// Checks a call whose target is a distinct type or a built-in scalar type name.
    ///
    /// Returns `None` for other type targets so the caller reports them as not callable.
    pub(in crate::check::expr) fn try_check_type_conversion(
        &mut self,
        call_key: usize,
        name: &str,
        symbol_ty: &Ty,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        let target = match self.resolve_visible_type(symbol_ty) {
            Ty::Named(type_name) => scalar_type_named(&type_name)?,
            Ty::Distinct(distinct) => Ty::Distinct(distinct),
            scalar @ (Ty::Integer | Ty::Real | Ty::String | Ty::Boolean) => scalar,
            _ => return None,
        };
        let Some(argument) = self.single_conversion_argument(name, args, span) else {
            return Some(target);
        };
        let argument_ty = self.check_expr(argument);
        let argument_ty = self.resolve_visible_type(&argument_ty);
        if argument_ty.is_error() {
            self.distinct_conversions.insert(call_key);
            return Some(target);
        }
        let valid = match &target {
            Ty::Distinct(distinct) => self.check_wrap(distinct, &argument_ty, argument.span()),
            scalar => self.check_unwrap(name, scalar, &argument_ty, argument.span()),
        };
        if valid {
            self.distinct_conversions.insert(call_key);
        }
        Some(target)
    }

    /// Names the explicit conversion for a mismatch between a distinct type and another type.
    pub(in crate::check) fn distinct_conversion_hint(expected: &Ty, actual: &Ty) -> Option<String> {
        match (expected, actual) {
            (Ty::Distinct(target), Ty::Distinct(source)) => Some(format!(
                "Distinct types are not interchangeable. Convert explicitly, for example `{}({}(Value))`.",
                target.name, source.underlying
            )),
            (Ty::Distinct(target), _) if target.underlying.assignment_compatible_with(actual) => {
                Some(format!(
                    "Distinct types are not converted implicitly. Wrap the value explicitly, for example `{}(Value)`.",
                    target.name
                ))
            }
            (_, Ty::Distinct(source))
                if expected.assignment_compatible_with(&source.underlying) =>
            {
                Some(format!(
                    "Distinct types are not converted implicitly. Unwrap the value explicitly, for example `{}(Value)`.",
                    source.underlying
                ))
            }
            _ => None,
        }
    }

    /// Rejects a distinct value passed where only its underlying value is accepted.
    pub(in crate::check) fn reject_implicit_distinct_unwrap(
        &mut self,
        routine: &str,
        ty: &Ty,
        span: Span,
    ) {
        if let Ty::Distinct(distinct) = self.resolve_visible_type(ty) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{routine}` does not accept distinct type `{}` implicitly",
                    distinct.name
                ),
                format!(
                    "Unwrap the value explicitly, for example `{}(Value)`.",
                    distinct.underlying
                ),
                span,
            );
        }
    }

    /// Accepts exactly one positional value argument.
    fn single_conversion_argument<'a>(
        &mut self,
        name: &str,
        args: &'a [Expr],
        span: Span,
    ) -> Option<&'a Expr> {
        if args.len() != 1 {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Conversion `{name}(...)` expects 1 argument, got {}",
                    args.len()
                ),
                format!("Pass exactly one value, for example `{name}(Value)`."),
                span,
            );
            self.check_args_only(args);
            return None;
        }
        let argument = &args[0];
        if matches!(
            argument,
            Expr::NamedArgument { .. } | Expr::VarArgument { .. }
        ) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Conversion `{name}(...)` takes one positional value"),
                format!("Write `{name}(Value)` without a parameter name or `var` marker."),
                argument.span(),
            );
            self.check_args_only(args);
            return None;
        }
        Some(argument)
    }

    /// Wraps a value of the exact underlying type, or keeps a value of the same distinct type.
    fn check_wrap(&mut self, target: &DistinctTy, argument: &Ty, span: Span) -> bool {
        match argument {
            Ty::Distinct(source) if source.same_declaration(target) => true,
            Ty::Distinct(source) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Cannot convert distinct type `{}` directly to distinct type `{}`",
                        source.name, target.name
                    ),
                    format!(
                        "Unwrap the value first, for example `{}({}(Value))`.",
                        target.name, source.underlying
                    ),
                    span,
                );
                false
            }
            _ if target.underlying.assignment_compatible_with(argument) => true,
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Cannot convert `{argument}` to distinct type `{}`",
                        target.name
                    ),
                    format!(
                        "`{}(...)` takes a value of its underlying type `{}`.",
                        target.name, target.underlying
                    ),
                    span,
                );
                false
            }
        }
    }

    /// Unwraps a distinct value whose underlying type is exactly `target`.
    fn check_unwrap(&mut self, name: &str, target: &Ty, argument: &Ty, span: Span) -> bool {
        match argument {
            Ty::Distinct(source) if target.assignment_compatible_with(&source.underlying) => true,
            Ty::Distinct(source) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Cannot unwrap distinct type `{}` to `{target}`: its underlying type is `{}`",
                        source.name, source.underlying
                    ),
                    format!(
                        "Unwrap with `{}(Value)` first. {SCALAR_CONVERSION_HINT}",
                        source.underlying
                    ),
                    span,
                );
                false
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "`{name}(...)` only unwraps a distinct type value, but the argument has type `{argument}`"
                    ),
                    SCALAR_CONVERSION_HINT,
                    span,
                );
                false
            }
        }
    }
}

/// Maps the primitive type symbols registered at the program root.
fn scalar_type_named(name: &str) -> Option<Ty> {
    [
        ("integer", Ty::Integer),
        ("real", Ty::Real),
        ("string", Ty::String),
        ("boolean", Ty::Boolean),
    ]
    .into_iter()
    .find_map(|(spelling, ty)| name.eq_ignore_ascii_case(spelling).then_some(ty))
}
