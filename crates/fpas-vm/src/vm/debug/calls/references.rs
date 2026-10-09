//! Detached writable designators and exact `var` argument types.
//! See `docs/pascal/language/functions/var-parameters.md` and `docs/pascal/tools/debugger.md`.

use std::sync::{Arc, Mutex};

use fpas_bytecode::{ReferenceRoot, ReferenceStep, Value, VariableReference};

use super::detach::error;
use super::execute::CallSandbox;
use crate::vm::debug::inspection::{InspectionSnapshot, MutationPath, MutationRoot};
use crate::vm::debug::mutation::{DebugAssignmentTarget, target_with_value};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

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
            MutationRoot::Global(index) => {
                ReferenceRoot::Global(u32::try_from(*index).map_err(|_| {
                    error(
                        DebugErrorKind::EvaluationType,
                        "debug reference global index overflows",
                        "Rebuild the executable.",
                    )
                })?)
            }
            MutationRoot::ClosureCell(cell) => {
                let Value::Cell(detached) = self.detacher.detach(&Value::Cell(Arc::clone(cell)))?
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
                    self.local_cells
                        .insert(*register, Arc::new(Mutex::new(detached)));
                }
                ReferenceRoot::Cell(Arc::clone(&self.local_cells[register]))
            }
        };
        let detached_root = match &root {
            ReferenceRoot::Global(index) => self
                .globals
                .try_read()
                .ok()
                .and_then(|globals| globals.get(*index as usize).cloned().flatten()),
            ReferenceRoot::Cell(cell) => cell.try_lock().ok().map(|value| value.clone()),
        }
        .ok_or_else(|| {
            error(
                DebugErrorKind::UnavailableValue,
                "detached reference root is unavailable",
                "Retry at a stable stop.",
            )
        })?;
        let current = crate::vm::debug::mutation::resolve_value_path(&detached_root, &target.path)
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
        (ReferenceRoot::Cell(left), ReferenceRoot::Cell(right)) => Arc::ptr_eq(left, right),
        (ReferenceRoot::Global(left), ReferenceRoot::Global(right)) => left == right,
        _ => false,
    }
}
