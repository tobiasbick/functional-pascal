//! Validation for source-level debugger metadata.

use std::collections::HashSet;

use crate::validate::{EntityKind, ValidationError, ValidationErrorKind, function_error};
use crate::{CaptureKind, Function, Program};

pub(super) fn validate_function(
    program: &Program,
    function: &Function,
) -> Result<(), ValidationError> {
    let mut scopes = HashSet::with_capacity(function.debug.scopes.len());
    for (index, scope) in function.debug.scopes.iter().enumerate() {
        if usize::try_from(scope.id).ok() != Some(index) || !scopes.insert(scope.id) {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::DuplicateId {
                    entity: EntityKind::DebugScope,
                    id: scope.id,
                },
            ));
        }
        match (scope.id, scope.parent) {
            (0, None) => {}
            (0, Some(parent)) | (_, Some(parent)) if parent >= scope.id => {
                return Err(unknown_scope(function, parent));
            }
            (_, None) => return Err(unknown_scope(function, scope.id)),
            _ => {}
        }
    }

    for binding in &function.debug.bindings {
        if function.local(binding.local).is_none() {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Local,
                    id: binding.local.get(),
                },
            ));
        }
        if program.ty(binding.ty).is_none() {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Type,
                    id: binding.ty.get(),
                },
            ));
        }
        validate_scope(function, binding.scope)?;
    }

    validate_capture_provenance(program, function)?;

    let mut points = HashSet::with_capacity(function.debug.sequence_points.len());
    for point in &function.debug.sequence_points {
        let Some(block) = function.block(point.block) else {
            return Err(function_error(
                function.id,
                Some(point.block),
                Some(point.instruction),
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Block,
                    id: point.block.get(),
                },
            ));
        };
        if point.instruction >= block.instructions.len() {
            return Err(function_error(
                function.id,
                Some(point.block),
                Some(point.instruction),
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Value,
                    id: u32::try_from(point.instruction).unwrap_or(u32::MAX),
                },
            ));
        }
        if !points.insert((point.block, point.instruction)) {
            return Err(function_error(
                function.id,
                Some(point.block),
                Some(point.instruction),
                ValidationErrorKind::DuplicateId {
                    entity: EntityKind::Value,
                    id: u32::try_from(point.instruction).unwrap_or(u32::MAX),
                },
            ));
        }
        validate_scope(function, point.scope)?;
    }
    Ok(())
}

fn validate_capture_provenance(
    program: &Program,
    function: &Function,
) -> Result<(), ValidationError> {
    let sources = &function.debug.capture_sources;
    if function.captures.is_empty() {
        if function.debug.lexical_owner.is_some() || !sources.is_empty() {
            return Err(provenance(
                function,
                "zero-capture functions must omit lexical owner and capture sources",
                sources.len() as u32,
            ));
        }
        return Ok(());
    }
    let Some(owner_id) = function.debug.lexical_owner else {
        return Err(provenance(
            function,
            "capturing functions must record a lexical owner",
            function.id.get(),
        ));
    };
    let Some(owner) = program.function(owner_id) else {
        return Err(function_error(
            function.id,
            None,
            None,
            ValidationErrorKind::UnknownId {
                entity: EntityKind::Function,
                id: owner_id.get(),
            },
        ));
    };
    if sources.len() != function.captures.len() {
        return Err(function_error(
            function.id,
            None,
            None,
            ValidationErrorKind::ClosureCaptureCount {
                expected: function.captures.len(),
                actual: sources.len(),
            },
        ));
    }
    for (index, (declaration, source)) in function.captures.iter().zip(sources).enumerate() {
        if source.kind != declaration.kind || source.ty != declaration.ty {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::ClosureCaptureType {
                    index,
                    expected: declaration.ty.get(),
                    actual: source.ty.get(),
                },
            ));
        }
        let binding_index = usize::try_from(source.binding.get()).unwrap_or(usize::MAX);
        let Some(binding) = owner.debug.bindings.get(binding_index) else {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::DebugBinding,
                    id: source.binding.get(),
                },
            ));
        };
        if binding.ty != source.ty {
            return Err(provenance(
                function,
                "capture source type must match the owner binding type",
                source.ty.get(),
            ));
        }
        match source.kind {
            CaptureKind::Value if binding.cell_backed => {
                return Err(provenance(
                    function,
                    "value captures cannot refer to a cell-backed owner binding",
                    source.binding.get(),
                ));
            }
            CaptureKind::Cell | CaptureKind::EnclosingCell if !binding.cell_backed => {
                return Err(provenance(
                    function,
                    "cell captures must refer to a cell-backed owner binding",
                    source.binding.get(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn provenance(function: &Function, reason: &'static str, actual: u32) -> ValidationError {
    function_error(
        function.id,
        None,
        None,
        ValidationErrorKind::CaptureProvenance { reason, actual },
    )
}

fn validate_scope(function: &Function, scope: u32) -> Result<(), ValidationError> {
    if function
        .debug
        .scopes
        .get(scope as usize)
        .is_some_and(|candidate| candidate.id == scope)
    {
        return Ok(());
    }
    Err(unknown_scope(function, scope))
}

fn unknown_scope(function: &Function, scope: u32) -> ValidationError {
    function_error(
        function.id,
        None,
        None,
        ValidationErrorKind::UnknownId {
            entity: EntityKind::DebugScope,
            id: scope,
        },
    )
}
