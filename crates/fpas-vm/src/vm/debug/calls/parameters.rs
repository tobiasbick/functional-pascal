//! Parameter modes and storage identity checked before starting a debug callee.
//! See `docs/pascal/language/functions/var-parameters.md` and `docs/pascal/tools/debugger.md`.

use std::sync::Arc;

use fpas_bytecode::{DebugBindingKind, DebugType, FunctionInfo, Value};

use super::detach::error;
use super::execute::CallSandbox;
use super::references::same_root;
use crate::vm::debug::mutation::portable_type::{TypeLimitWording, require_equal_bounded};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl CallSandbox {
    /// Validates explicit argument modes, exact referent types, and distinct `var` roots.
    pub(super) fn require_parameters(
        &self,
        function: &FunctionInfo,
        arguments: &[Value],
        display_name: &str,
    ) -> Result<(), DebugSessionError> {
        let image = self.executable.executable();
        let mut parameters = function
            .debug
            .bindings
            .iter()
            .filter(|binding| binding.kind == DebugBindingKind::Parameter)
            .collect::<Vec<_>>();
        parameters.sort_by_key(|binding| binding.register.get());
        if parameters.len() != arguments.len() {
            return Err(error(
                DebugErrorKind::UnavailableValue,
                "debug call requires complete parameter type and mode metadata",
                "Rebuild the executable with the current compiler.",
            ));
        }
        let mut roots: Vec<&fpas_bytecode::VariableReference> = Vec::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            let name = image.strings.get(parameter.name).unwrap_or("<parameter>");
            match (image.debug_types.get(parameter.ty.get() as usize), argument) {
                (Some(DebugType::Reference(expected)), Value::Reference(reference)) => {
                    let Some(actual) = self.reference_types.get(&(Arc::as_ptr(reference) as usize))
                    else {
                        return Err(error(
                            DebugErrorKind::EvaluationType,
                            "debug `var` argument is not owned by the detached sandbox",
                            "Pass a visible writable designator with explicit `var`.",
                        ));
                    };
                    require_equal_bounded(
                        &image.debug_types,
                        *expected,
                        *actual,
                        self.limits.max_depth,
                        self.limits.max_detached_values,
                        TypeLimitWording {
                            subject: "var parameter type",
                            assignment: "var argument",
                        },
                    )
                    .map_err(|failure| {
                        error(
                            DebugErrorKind::EvaluationType,
                            failure.message,
                            failure.hint,
                        )
                    })?;
                    if roots.iter().any(|previous| same_root(previous, reference)) {
                        return Err(error(
                            DebugErrorKind::EvaluationType,
                            format!(
                                "debug callable `{display_name}` passes the same root to multiple `var` parameters"
                            ),
                            "Use distinct writable roots, including for different fields or elements.",
                        ));
                    }
                    roots.push(reference.as_ref());
                }
                (Some(DebugType::Reference(_)), _) => {
                    return Err(error(
                        DebugErrorKind::EvaluationType,
                        format!(
                            "debug callable `{display_name}` declares a `var` parameter `{name}` requiring an explicit `var` argument"
                        ),
                        format!(
                            "Pass writable storage explicitly, for example `{display_name}(var Item)`."
                        ),
                    ));
                }
                (_, Value::Reference(_)) => {
                    return Err(error(
                        DebugErrorKind::EvaluationType,
                        format!("debug callable `{display_name}` parameter `{name}` takes a value"),
                        "Remove `var` from the argument of a value parameter.",
                    ));
                }
                (Some(DebugType::Dynamic), _)
                    if image
                        .strings
                        .get(parameter.type_name)
                        .is_some_and(|name| name.starts_with("var ")) =>
                {
                    return Err(error(
                        DebugErrorKind::UnavailableValue,
                        "debug `var` parameter lacks portable reference metadata",
                        "Rebuild with the current compiler.",
                    ));
                }
                _ => crate::vm::debug::mutation::validate_call_value(
                    &self.executable,
                    parameter.ty,
                    argument,
                    self.limits.max_depth,
                )
                .map_err(|failure| {
                    error(
                        DebugErrorKind::EvaluationType,
                        format!(
                            "debug callable `{display_name}` parameter `{name}`: {}",
                            failure.message
                        ),
                        failure.hint,
                    )
                })?,
            }
        }
        Ok(())
    }
}
