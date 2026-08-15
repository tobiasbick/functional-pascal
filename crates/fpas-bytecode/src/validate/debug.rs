//! Validation for executable debugger scopes, bindings, and sequence points.

use crate::{DebugCaptureKind, DebugSourceLocation, FunctionId};

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
        validate_capture_provenance(executable, function_id)?;
    }
    Ok(())
}

fn validate_capture_provenance(
    executable: &crate::Executable,
    function_id: FunctionId,
) -> Result<(), ValidationError> {
    let function = &executable.functions[usize::from(function_id.get())];
    let sources = &function.debug.capture_sources;
    if function.capture_count == 0 {
        if function.debug.lexical_owner.is_some() || !sources.is_empty() {
            return Err(provenance(
                executable,
                function_id,
                "zero-capture functions must omit lexical owner and capture sources",
                sources.len() as u32,
                0,
            ));
        }
        return Ok(());
    }
    let Some(owner_id) = function.debug.lexical_owner else {
        return Err(provenance(
            executable,
            function_id,
            "capturing functions must record a lexical owner",
            u32::from(function_id.get()),
            1,
        ));
    };
    let owner_index = usize::from(owner_id.get());
    let Some(owner) = executable.functions.get(owner_index) else {
        return Err(provenance(
            executable,
            function_id,
            "lexical owner function is out of bounds",
            u32::from(owner_id.get()),
            executable.functions.len() as u32,
        ));
    };
    if sources.len() != usize::from(function.capture_count) {
        return Err(provenance(
            executable,
            function_id,
            "capture source count must equal the function capture count",
            sources.len() as u32,
            u32::from(function.capture_count),
        ));
    }
    for source in sources {
        let binding_index = usize::try_from(source.binding.get()).unwrap_or(usize::MAX);
        let Some(binding) = owner.debug.bindings.get(binding_index) else {
            return Err(provenance(
                executable,
                function_id,
                "capture source binding is out of bounds on the lexical owner",
                source.binding.get(),
                owner.debug.bindings.len() as u32,
            ));
        };
        if executable
            .debug_types
            .get(source.ty.get() as usize)
            .is_none()
        {
            return Err(ValidationError::function(
                executable,
                function_id,
                ValidationErrorKind::TableReference {
                    table: "debug types",
                    operand: "capture source type",
                    actual: u64::from(source.ty.get()),
                    length: executable.debug_types.len(),
                },
            ));
        }
        if binding.ty != source.ty {
            return Err(provenance(
                executable,
                function_id,
                "capture source type must match the owner binding type",
                source.ty.get(),
                binding.ty.get(),
            ));
        }
        match source.kind {
            DebugCaptureKind::Value if binding.cell_backed || binding.hidden => {
                return Err(provenance(
                    executable,
                    function_id,
                    "value captures cannot refer to a hidden or cell-backed owner binding",
                    source.binding.get(),
                    0,
                ));
            }
            DebugCaptureKind::Cell | DebugCaptureKind::EnclosingCell if !binding.cell_backed => {
                return Err(provenance(
                    executable,
                    function_id,
                    "cell captures must refer to a cell-backed owner binding",
                    source.binding.get(),
                    1,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn provenance(
    executable: &crate::Executable,
    function: FunctionId,
    reason: &'static str,
    actual: u32,
    expected: u32,
) -> ValidationError {
    ValidationError::function(
        executable,
        function,
        ValidationErrorKind::DebugCaptureProvenance {
            reason,
            actual,
            expected,
        },
    )
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
