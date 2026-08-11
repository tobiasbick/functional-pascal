//! Generation-scoped frame, scope, variable, evaluation, and mutation handles.

use std::collections::HashSet;

use fpas_bytecode::Value;

use super::model::{DebugFrame, DebugScope, DebugVariable, Paginated};
use super::render::{self, RetainedValue};
use super::snapshot::{HandleEntry, InspectionSnapshot, item_id};
use super::targets::{MutationAccess, MutationTarget};
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl InspectionSnapshot {
    pub(in crate::vm::debug) fn validate_evaluation_frame(
        &self,
        frame_id: Option<u64>,
    ) -> Result<(), DebugSessionError> {
        if frame_id.is_none()
            || self
                .frames
                .iter()
                .any(|frame| Some(frame.frame.id) == frame_id)
        {
            return Ok(());
        }
        let frame_id = frame_id.unwrap_or_default();
        Err(DebugSessionError {
            kind: DebugErrorKind::UnknownFrame,
            message: format!("debug frame {frame_id} is unknown or expired"),
            hint: "Request stack frames again for the current stop.".to_string(),
        })
    }

    pub(in crate::vm::debug) fn stack(
        &self,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugFrame>, DebugSessionError> {
        self.check_page("stack", count, self.limits.max_frames)?;
        let items = self
            .frames
            .iter()
            .skip(start)
            .take(count)
            .map(|frame| frame.frame.clone())
            .collect();
        Ok(Paginated {
            items,
            total: self.total_frames,
        })
    }

    pub(in crate::vm::debug) fn scopes(
        &self,
        frame_id: u64,
    ) -> Result<Vec<DebugScope>, DebugSessionError> {
        self.frames
            .iter()
            .find(|frame| frame.frame.id == frame_id)
            .map(|frame| frame.scopes.clone())
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {frame_id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })
    }

    pub(in crate::vm::debug) fn variables(
        &mut self,
        reference: u64,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugVariable>, DebugSessionError> {
        self.check_page("variables", count, self.limits.max_children)?;
        let Some(handle_index) = self.handles.iter().position(|entry| entry.id == reference) else {
            return Err(DebugSessionError {
                kind: DebugErrorKind::UnknownVariablesReference,
                message: format!("debug variables reference {reference} is unknown or expired"),
                hint: "Request scopes or parent variables again for the current stop.".to_string(),
            });
        };
        let total = self.handles[handle_index].values.len();
        let retained = self.handles[handle_index]
            .values
            .iter()
            .skip(start)
            .take(count)
            .cloned()
            .collect::<Vec<_>>();
        let mut items = Vec::with_capacity(retained.len());
        let mut output_bytes = 0_usize;
        for (relative_index, retained) in retained.into_iter().enumerate() {
            let absolute_index = start.saturating_add(relative_index);
            let rendered = render::render_with_executable(
                &retained,
                self.limits,
                Some(self.executable.executable()),
            );
            let variables_reference = if rendered.children.is_empty() {
                0
            } else if let Some(existing) = self.child_handles.get(&(reference, absolute_index)) {
                *existing
            } else {
                let handle = self.allocate_handle(rendered.children)?;
                self.child_handles
                    .insert((reference, absolute_index), handle);
                handle
            };
            let variable = DebugVariable {
                name: retained.name,
                value: rendered.summary,
                type_name: rendered.type_name,
                variables_reference,
                named_variables: rendered.named_children,
                indexed_variables: rendered.indexed_children,
                presentation_hint: rendered.presentation_hint,
            };
            let size = variable.name.len()
                + variable.value.len()
                + variable.type_name.len()
                + variable.presentation_hint.as_ref().map_or(0, String::len);
            if output_bytes.saturating_add(size) > self.limits.max_output_bytes {
                break;
            }
            output_bytes += size;
            items.push(variable);
        }
        Ok(Paginated { items, total })
    }

    pub(in crate::vm::debug) fn resolve_evaluation_name(
        &self,
        frame_id: Option<u64>,
        name: &str,
    ) -> Result<Value, DebugSessionError> {
        let frame_values = match frame_id {
            Some(frame_id) => Some(
                self.frames
                    .iter()
                    .find(|frame| frame.frame.id == frame_id)
                    .map(|frame| frame.evaluation_values.as_slice())
                    .ok_or_else(|| DebugSessionError {
                        kind: DebugErrorKind::UnknownFrame,
                        message: format!("debug frame {frame_id} is unknown or expired"),
                        hint: "Request stack frames again for the current stop.".to_string(),
                    })?,
            ),
            None => None,
        };
        resolve_name(frame_values, &self.globals, name)
    }

    pub(in crate::vm::debug) fn resolve_mutation_target(
        &self,
        reference: u64,
        name: &str,
    ) -> Result<MutationTarget, DebugSessionError> {
        if (reference >> 32) as u32 != self.generation {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableTargetExpired,
                message: format!(
                    "debug variable target `{name}` belongs to an expired stop snapshot"
                ),
                hint: "Request scopes and variables again for the current stop.".to_string(),
            });
        }
        let retained = self
            .handles
            .iter()
            .find(|entry| entry.id == reference)
            .and_then(|entry| {
                entry
                    .values
                    .iter()
                    .find(|value| value.name.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::VariableTargetUnknown,
                message: format!("debug variable target `{name}` does not exist"),
                hint: "Request the container variables again and use a returned child name."
                    .to_string(),
            })?;
        if retained.value.is_none() {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableUninitialized,
                message: format!("debug variable target `{name}` is uninitialized"),
                hint: "Stop after the binding has received a value.".to_string(),
            });
        }
        match &retained.mutation {
            MutationAccess::Writable(target) => Ok(target.clone()),
            MutationAccess::NotMutable => Err(DebugSessionError {
                kind: DebugErrorKind::VariableNotMutable,
                message: format!("debug variable target `{name}` is not mutable"),
                hint: "Select a source-declared mutable binding or descendant.".to_string(),
            }),
            MutationAccess::Unsupported => Err(DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message: format!("debug variable target `{name}` is not assignable"),
                hint: "Only records, array elements, and existing dictionary values below a mutable root are assignable."
                    .to_string(),
            }),
            MutationAccess::Unavailable => Err(DebugSessionError {
                kind: DebugErrorKind::VariableUnavailable,
                message: format!("debug variable target `{name}` is unavailable"),
                hint: "Retry at a stable stop after the live storage becomes available."
                    .to_string(),
            }),
        }
    }

    pub(in crate::vm::debug) fn retain_evaluation_result(
        &mut self,
        value: Value,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        let retained = RetainedValue {
            name: "$result".to_string(),
            type_name: value.type_name().to_string(),
            value: Some(value),
            presentation_hint: None,
            depth: 0,
            visited_cells: HashSet::new(),
            debug_type: None,
            mutation: MutationAccess::NotMutable,
        };
        let rendered = render::render_with_executable(
            &retained,
            self.limits,
            Some(self.executable.executable()),
        );
        let output_bytes = rendered
            .summary
            .len()
            .saturating_add(rendered.type_name.len());
        if output_bytes > limits.max_output_bytes {
            return Err(DebugSessionError {
                kind: DebugErrorKind::EvaluationLimit,
                message: format!(
                    "debug expression result uses {output_bytes} bytes, exceeding limit {}",
                    limits.max_output_bytes
                ),
                hint: "Evaluate a smaller value or expand it through Variables.".to_string(),
            });
        }
        let variables_reference = if rendered.children.is_empty() {
            0
        } else {
            self.allocate_handle(rendered.children)?
        };
        Ok(DebugEvaluateResult {
            value: rendered.summary,
            type_name: rendered.type_name,
            variables_reference,
            named_variables: rendered.named_children,
            indexed_variables: rendered.indexed_children,
        })
    }

    pub(super) fn allocate_handle(
        &mut self,
        values: Vec<RetainedValue>,
    ) -> Result<u64, DebugSessionError> {
        if self.handles.len() >= self.limits.max_handles {
            return Err(DebugSessionError {
                kind: DebugErrorKind::InspectionLimit,
                message: "debug variable handle limit reached".to_string(),
                hint: "Reduce aggregate expansion or increase the configured handle limit."
                    .to_string(),
            });
        }
        let id = item_id(self.generation, self.handles.len());
        self.handles.push(HandleEntry { id, values });
        Ok(id)
    }

    fn check_page(
        &self,
        operation: &'static str,
        count: usize,
        maximum: usize,
    ) -> Result<(), DebugSessionError> {
        if count <= maximum {
            return Ok(());
        }
        Err(DebugSessionError {
            kind: DebugErrorKind::InspectionLimit,
            message: format!("debug {operation} page count {count} exceeds limit {maximum}"),
            hint: format!("Request at most {maximum} items and use pagination."),
        })
    }
}

fn resolve_name(
    frame_values: Option<&[RetainedValue]>,
    globals: &[RetainedValue],
    name: &str,
) -> Result<Value, DebugSessionError> {
    let retained = frame_values
        .into_iter()
        .flatten()
        .chain(globals)
        .find(|value| value.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::UnknownName,
            message: format!("debug expression name `{name}` is not visible"),
            hint: "Use a parameter, local, capture, or global visible at the selected frame."
                .to_string(),
        })?;
    retained.value.clone().ok_or_else(|| DebugSessionError {
        kind: DebugErrorKind::UninitializedValue,
        message: format!("debug expression name `{name}` is uninitialized"),
        hint: "Stop after the binding has received a value.".to_string(),
    })
}
