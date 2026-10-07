//! Mapping of named call arguments to declared parameters and variant fields.
//!
//! **Documentation:** `docs/pascal/language/functions/parameters.md`

use super::super::Checker;
use super::CallTarget;
use crate::types::{ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_INVALID_NAMED_ARGUMENT, SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED};
use fpas_parser::Expr;

impl Checker {
    /// Returns argument values in parameter order.
    ///
    /// Positional calls keep their written order. A fully named call is mapped
    /// to the declared parameter names (case-insensitive) and its order is
    /// recorded for lowering, which evaluates the arguments in written order.
    /// Returns `None` after reporting an invalid mapping and checking the values.
    pub(in crate::check) fn order_call_arguments<'a>(
        &mut self,
        name: &str,
        target: CallTarget,
        params: &[ParamTy],
        variadic: bool,
        args: &[&'a Expr],
    ) -> Option<Vec<&'a Expr>> {
        if !args.iter().any(|arg| arg.argument_name().is_some()) {
            return Some(args.to_vec());
        }
        let hint = match target {
            CallTarget::Routine if variadic => Some(format!(
                "`{name}` takes a variable number of arguments; pass them by position."
            )),
            CallTarget::Routine | CallTarget::EnumVariant => None,
            CallTarget::FunctionValue => Some(format!(
                "`{name}` is a function value; call it positionally, for example `{name}(…)`. Function types do not carry parameter names."
            )),
            CallTarget::ReceiverCall => Some(format!(
                "Receiver calls `.{name}(…)` take positional arguments. Call the routine directly to name its parameters."
            )),
        };
        if let Some(hint) = hint {
            self.reject_named_arguments(args.iter().copied(), name, &hint);
            self.check_argument_values(args);
            return None;
        }

        let (noun, title, nouns) = if target == CallTarget::EnumVariant {
            ("field", "Field", "Fields")
        } else {
            ("parameter", "Parameter", "Parameters")
        };
        let mut slots: Vec<Option<usize>> = vec![None; params.len()];
        let mut valid = true;
        for (written, arg) in args.iter().enumerate() {
            let Expr::NamedArgument {
                name: arg_name,
                name_span,
                ..
            } = arg
            else {
                valid = false;
                continue;
            };
            let Some(index) = params
                .iter()
                .position(|param| param.name.eq_ignore_ascii_case(arg_name))
            else {
                self.error_with_code(
                    SEMA_INVALID_NAMED_ARGUMENT,
                    format!("`{name}` has no {noun} `{arg_name}`"),
                    format!("Expected {noun} names: {}.", parameter_list(params)),
                    *name_span,
                );
                valid = false;
                continue;
            };
            if slots[index].replace(written).is_some() {
                self.error_with_code(
                    SEMA_INVALID_NAMED_ARGUMENT,
                    format!(
                        "{title} `{}` of `{name}` is named more than once",
                        params[index].name
                    ),
                    format!("Pass each {noun} exactly once."),
                    *name_span,
                );
                valid = false;
            }
        }
        let missing = params
            .iter()
            .zip(&slots)
            .filter(|(_, slot)| slot.is_none())
            .map(|(param, _)| format!("`{}`", param.name))
            .collect::<Vec<_>>();
        if valid && !missing.is_empty() {
            let example = params
                .iter()
                .map(|param| format!("{} := …", param.name))
                .collect::<Vec<_>>()
                .join(", ");
            self.error_with_code(
                SEMA_INVALID_NAMED_ARGUMENT,
                format!("Named call to `{name}` is missing {}", missing.join(", ")),
                format!("{nouns} have no default values; pass every {noun}: `{name}({example})`."),
                args.last().map_or_else(|| args[0].span(), |arg| arg.span()),
            );
            valid = false;
        }
        if !valid {
            self.check_argument_values(args);
            return None;
        }

        let order = slots.into_iter().flatten().collect::<Vec<_>>();
        self.named_argument_orders
            .insert(Self::expr_lookup_key(args[0]), order.clone());
        Some(
            order
                .into_iter()
                .map(|written| args[written].argument_value())
                .collect(),
        )
    }

    /// Reports the first named argument of a call that accepts only positional
    /// arguments. Returns whether one was found.
    pub(crate) fn reject_named_arguments<'a>(
        &mut self,
        args: impl IntoIterator<Item = &'a Expr>,
        callee: &str,
        hint: &str,
    ) -> bool {
        let Some(Expr::NamedArgument {
            name, name_span, ..
        }) = args.into_iter().find(|arg| arg.argument_name().is_some())
        else {
            return false;
        };
        self.error_with_code(
            SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED,
            format!("`{callee}` does not accept named arguments such as `{name} := …`"),
            hint,
            *name_span,
        );
        true
    }

    /// Reports a named argument outside a call that supports it.
    pub(crate) fn check_misplaced_named_argument(&mut self, expr: &Expr) -> Ty {
        let Expr::NamedArgument {
            name,
            name_span,
            value,
            ..
        } = expr
        else {
            return Ty::Error;
        };
        self.error_with_code(
            SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED,
            format!("Named argument `{name} := …` is not supported here"),
            "Named arguments are valid in calls of declared routines, record methods, and enum variant constructors. Pass the value by position, for example `Ok(Value)`.",
            *name_span,
        );
        let _ = self.check_expr(value);
        Ty::Error
    }

    fn check_argument_values(&mut self, args: &[&Expr]) {
        for arg in args {
            let _ = self.check_expr(arg.argument_value());
        }
    }
}

fn parameter_list(params: &[ParamTy]) -> String {
    if params.is_empty() {
        return "none".to_string();
    }
    params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
