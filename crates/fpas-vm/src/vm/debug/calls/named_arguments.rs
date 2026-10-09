//! Parameter-name mapping after written-order evaluation.
//! See `docs/pascal/language/functions/parameters.md#named-arguments`.

use fpas_bytecode::{DebugBindingKind, Value, VerifiedExecutable};

use super::detach::error;
use super::execute::CallSandbox;
use super::resolution::{NamedTarget, resolve_named};
use crate::vm::debug::evaluation::DebugCallTarget;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl CallSandbox {
    /// Maps named arguments to declared parameters or enum fields.
    pub(super) fn order_named_arguments(
        &self,
        target: &DebugCallTarget,
        names: &[String],
        arguments: Vec<Value>,
    ) -> Result<Vec<Value>, DebugSessionError> {
        let (name, skip_receiver) = match target {
            DebugCallTarget::Named(name) => (name.clone(), false),
            DebugCallTarget::Method { receiver, name } => (self.member_name(receiver, name)?, true),
            _ => {
                return Err(error(
                    DebugErrorKind::EvaluationType,
                    "debug function values accept only positional arguments",
                    "Call function values positionally; their types do not retain parameter names.",
                ));
            }
        };
        let parameters = match resolve_named(&self.executable, &self.layouts, &name)? {
            NamedTarget::Function(function) => {
                parameter_names(&self.executable, function.get() as usize, skip_receiver)?
            }
            NamedTarget::EnumConstructor(layout) => layout.fields.clone(),
            NamedTarget::Intrinsic(_) => {
                return Err(error(
                    DebugErrorKind::EvaluationType,
                    format!("debug intrinsic `{name}` has no retained parameter-name metadata"),
                    "Call this intrinsic positionally.",
                ));
            }
        };
        order(&name, &parameters, names, arguments)
    }
}

fn parameter_names(
    executable: &VerifiedExecutable,
    function: usize,
    skip_receiver: bool,
) -> Result<Vec<String>, DebugSessionError> {
    let image = executable.executable();
    let info = &image.functions[function];
    let mut bindings = info
        .debug
        .bindings
        .iter()
        .filter(|binding| binding.kind == DebugBindingKind::Parameter)
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| binding.register.get());
    if bindings.len() != info.arity as usize {
        return Err(error(
            DebugErrorKind::UnavailableValue,
            "debug callable has incomplete declared parameter metadata",
            "Rebuild the program with current debugger metadata or call positionally.",
        ));
    }
    bindings
        .into_iter()
        .skip(usize::from(skip_receiver))
        .map(|binding| {
            image
                .strings
                .get(binding.name)
                .map(str::to_owned)
                .ok_or_else(|| {
                    error(
                        DebugErrorKind::UnavailableValue,
                        "debug callable has an unavailable parameter name",
                        "Rebuild the executable with the current compiler.",
                    )
                })
        })
        .collect()
}

/// Places each supplied value in its unique declared parameter slot.
pub(super) fn order(
    callable: &str,
    parameters: &[String],
    names: &[String],
    arguments: Vec<Value>,
) -> Result<Vec<Value>, DebugSessionError> {
    if names.len() != arguments.len() {
        return Err(error(
            DebugErrorKind::CallArity,
            "debug named-argument metadata does not match the supplied values",
            "Supply one name for every call argument.",
        ));
    }
    let mut ordered = vec![None; parameters.len()];
    for (name, value) in names.iter().zip(arguments) {
        let Some(index) = parameters
            .iter()
            .position(|parameter| parameter.eq_ignore_ascii_case(name))
        else {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!("debug callable `{callable}` has no parameter or field `{name}`"),
                format!("Declared names: {}.", parameters.join(", ")),
            ));
        };
        if ordered[index].replace(value).is_some() {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug callable `{callable}` names `{}` more than once",
                    parameters[index]
                ),
                "Pass each declared parameter or field exactly once.",
            ));
        }
    }
    ordered
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            value.ok_or_else(|| {
                error(
                    DebugErrorKind::CallArity,
                    format!(
                        "debug callable `{callable}` is missing `{}`",
                        parameters[index]
                    ),
                    "Pass every declared parameter or enum field exactly once.",
                )
            })
        })
        .collect()
}
