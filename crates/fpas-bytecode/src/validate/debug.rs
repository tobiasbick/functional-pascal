//! Validation for executable debugger scopes, bindings, and sequence points.

use crate::{DebugSourceLocation, FunctionId};

use super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_debug_info(executable: &crate::Executable) -> Result<(), ValidationError> {
    for (index, function) in executable.functions.iter().enumerate() {
        let function_id = FunctionId::try_from_index(index).map_err(|_| {
            ValidationError::executable(ValidationErrorKind::ResourceLimit {
                resource: "functions",
                actual: executable.functions.len(),
                maximum: crate::limits::MAX_FUNCTIONS,
            })
        })?;
        for (scope_index, scope) in function.debug.scopes.iter().enumerate() {
            let valid_id = usize::try_from(scope.id).ok() == Some(scope_index);
            let valid_parent = match (scope.id, scope.parent) {
                (0, None) => true,
                (_, Some(parent)) => parent < scope.id,
                _ => false,
            };
            if !valid_id || !valid_parent {
                return Err(scope_error(
                    executable,
                    function_id,
                    scope.parent.unwrap_or(scope.id),
                    function.debug.scopes.len(),
                ));
            }
        }
        for binding in &function.debug.bindings {
            validate_scope(executable, function_id, binding.scope)?;
            if binding.register.get() >= function.register_count {
                return Err(ValidationError::function(
                    executable,
                    function_id,
                    ValidationErrorKind::DebugBindingRegister {
                        actual: binding.register.get(),
                        registers: function.register_count,
                    },
                ));
            }
            if let Some(location) = binding.declaration {
                validate_location(executable, function_id, location)?;
            }
        }
        let mut previous = None;
        for point in &function.debug.sequence_points {
            let address = point.instruction.get();
            if !function.code.contains(point.instruction) {
                return Err(ValidationError::function(
                    executable,
                    function_id,
                    ValidationErrorKind::DebugSequenceAddress {
                        actual: address,
                        start: function.code.start.get(),
                        end: function.code.end.get(),
                    },
                ));
            }
            if previous.is_some_and(|previous| previous >= address) {
                return Err(ValidationError::function(
                    executable,
                    function_id,
                    ValidationErrorKind::DebugSequenceOrder {
                        previous: previous.unwrap_or(address),
                        actual: address,
                    },
                ));
            }
            validate_scope(executable, function_id, point.scope)?;
            validate_location(executable, function_id, point.location)?;
            previous = Some(address);
        }
    }
    Ok(())
}

fn validate_scope(
    executable: &crate::Executable,
    function: FunctionId,
    scope: u32,
) -> Result<(), ValidationError> {
    let scopes = executable.functions[usize::from(function.get())]
        .debug
        .scopes
        .len();
    if usize::try_from(scope)
        .ok()
        .is_some_and(|scope| scope < scopes)
    {
        Ok(())
    } else {
        Err(scope_error(executable, function, scope, scopes))
    }
}

fn validate_location(
    executable: &crate::Executable,
    function: FunctionId,
    location: DebugSourceLocation,
) -> Result<(), ValidationError> {
    if usize::try_from(location.source.get())
        .ok()
        .is_none_or(|source| source >= executable.source_map.sources.len())
    {
        return Err(ValidationError::function(
            executable,
            function,
            ValidationErrorKind::SourceReference {
                actual: location.source.get(),
                sources: executable.source_map.sources.len(),
            },
        ));
    }
    if location.line == 0 || location.column == 0 {
        return Err(ValidationError::function(
            executable,
            function,
            ValidationErrorKind::SourcePosition {
                line: location.line,
                column: location.column,
            },
        ));
    }
    Ok(())
}

fn scope_error(
    executable: &crate::Executable,
    function: FunctionId,
    actual: u32,
    scopes: usize,
) -> ValidationError {
    ValidationError::function(
        executable,
        function,
        ValidationErrorKind::DebugScope { actual, scopes },
    )
}
