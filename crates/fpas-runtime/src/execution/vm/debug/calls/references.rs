//! Detached writable designators and exact `var` argument types.
//! See `docs/pascal/language/functions/var-parameters.md` and `docs/pascal/tools/debugger.md`.

use std::sync::Arc;

use crate::{ReferenceRoot, ReferenceStep, Value, VariableReference};

use super::detach::error;
use super::execute::CallSandbox;
use crate::execution::vm::debug::inspection::{InspectionSnapshot, MutationPath, MutationRoot};
use crate::execution::vm::debug::mutation::{DebugAssignmentTarget, target_with_value};
use crate::execution::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl CallSandbox {
    /// Resolves initialized writable storage and copies its root into this sandbox.
    pub(super) fn reference(
        &mut self,
        inspection: &InspectionSnapshot,
        frame_id: Option<u64>,
        assignment: &DebugAssignmentTarget,
        indexes: &[Value],
    ) -> Result<Value, DebugSessionError> {
        let (target, current) =
            inspection.resolve_named_mutation_target(frame_id, &assignment.root)?;
        let root_value = current.ok_or_else(|| {
            error(
                DebugErrorKind::VariableUnavailable,
                "debug `var` argument requires initialized storage",
                "Initialize the variable before passing it by reference.",
            )
        })?;
        let root = match &target.root {
            MutationRoot::Global(index) => ReferenceRoot::Global {
                slot: u32::try_from(*index).map_err(|_| {
                    error(
                        DebugErrorKind::EvaluationType,
                        "debug reference global index overflows",
                        "Rebuild the executable.",
                    )
                })?,
                owner: self.detacher.domain.id().clone(),
            },
            MutationRoot::ClosureCell(cell) => {
                let Value::Cell(detached) = self.detacher.detach(&Value::Cell(cell.clone()))?
                else {
                    return Err(error(
                        DebugErrorKind::UnavailableValue,
                        "debug reference cell is unavailable",
                        "Retry at a stable stop.",
                    ));
                };
                ReferenceRoot::Cell(detached)
            }
            MutationRoot::FrameRegister(register) => {
                if !self.local_cells.contains_key(register) {
                    let detached = self.detacher.detach(&root_value)?;
                    self.local_cells.insert(
                        *register,
                        self.detacher.domain.new_cell(detached).map_err(|_| {
                            error(
                                DebugErrorKind::UnavailableValue,
                                "detached reference has foreign storage",
                                "Use values from the selected task.",
                            )
                        })?,
                    );
                    self.temporary_roots
                        .push(Value::Cell(self.local_cells[register].clone()));
                }
                ReferenceRoot::Cell(self.local_cells[register].clone())
            }
        };
        let detached_root = match &root {
            ReferenceRoot::Global { slot: index, .. } => self
                .globals
                .try_read()
                .ok()
                .and_then(|globals| globals.get(*index as usize).cloned().flatten()),
            ReferenceRoot::Cell(cell) => cell.try_read(&self.detacher.domain).ok(),
        }
        .ok_or_else(|| {
            error(
                DebugErrorKind::UnavailableValue,
                "detached reference root is unavailable",
                "Retry at a stable stop.",
            )
        })?;
        let current =
            crate::execution::vm::debug::mutation::resolve_value_path(&detached_root, &target.path)
                .cloned()
                .ok_or_else(|| {
                    error(
                        DebugErrorKind::VariablePathUnsupported,
                        "debug reference path no longer exists in the detached sandbox",
                        "Use a designator that remains valid after preceding argument evaluations.",
                    )
                })?;
        let (target, _) = target_with_value(
            self.executable.executable(),
            assignment,
            target,
            current,
            indexes,
        )?;
        let path = target
            .path
            .iter()
            .map(|component| match component {
                MutationPath::RecordField(index) => {
                    u16::try_from(*index).ok().map(ReferenceStep::Field)
                }
                MutationPath::ArrayIndex(index) => {
                    i64::try_from(*index).ok().map(ReferenceStep::Element)
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                error(
                    DebugErrorKind::VariablePathUnsupported,
                    "debug `var` arguments support only stored record fields and array elements",
                    "Pass a mutable binding, record field, or array element with explicit `var`.",
                )
            })?;
        let reference = Arc::new(VariableReference { root, path });
        self.reference_types
            .insert(Arc::as_ptr(&reference) as usize, target.expected_type);
        Ok(Value::Reference(reference))
    }

    /// Keeps references owned by this sandbox while deeply detaching ordinary values.
    pub(super) fn detach_argument(&mut self, value: &Value) -> Result<Value, DebugSessionError> {
        if let Value::Reference(reference) = value
            && self
                .reference_types
                .contains_key(&(Arc::as_ptr(reference) as usize))
        {
            return Ok(value.clone());
        }
        self.detacher.detach(value)
    }
}

/// Whether two arguments refer to the same storage root, even through different paths.
pub(super) fn same_root(left: &VariableReference, right: &VariableReference) -> bool {
    match (&left.root, &right.root) {
        (ReferenceRoot::Cell(left), ReferenceRoot::Cell(right)) => left.ptr_eq(right),
        (
            ReferenceRoot::Global {
                slot: left,
                owner: left_owner,
            },
            ReferenceRoot::Global {
                slot: right,
                owner: right_owner,
            },
        ) => left == right && left_owner == right_owner,
        _ => false,
    }
}
