//! References created for `var` arguments: reads and writes reach the caller's variable.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use std::sync::Arc;

use fpas_bytecode::{
    AbcOperands, AbxOperands, ReferenceRoot, ReferenceStep, Value, VariableReference,
};
use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

use super::super::VmError;
use super::super::execute::scalar::register;
use super::super::worker::Worker;

/// A referenced path that no longer matches the root value.
enum PathFailure {
    NotRecord(&'static str),
    NotArray(&'static str),
    MissingField(u16),
    NegativeIndex(i64),
    IndexOutOfBounds { index: i64, len: usize },
}

impl Worker {
    /// Creates a reference to the variable held by a capture cell.
    pub fn make_cell_reference(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let cell = match self.read(register(o.b)?)? {
            Value::Cell(cell) => cell.clone(),
            other => return Err(self.type_mismatch("cell", other)),
        };
        self.write(
            register(o.a)?,
            Value::Reference(Arc::new(VariableReference {
                root: ReferenceRoot::Cell(cell),
                path: Vec::new(),
            })),
        )
    }

    /// Creates a reference to a mutable global slot.
    pub fn make_global_reference(&mut self, o: AbxOperands) -> Result<(), VmError> {
        self.write(
            register(o.a)?,
            Value::Reference(Arc::new(VariableReference {
                root: ReferenceRoot::Global(o.bx),
                path: Vec::new(),
            })),
        )
    }

    /// Narrows a reference to one positional record field.
    pub fn reference_field(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let reference = self.reference_operand(o.b)?;
        self.write(
            register(o.a)?,
            Value::Reference(Arc::new(reference.narrowed(ReferenceStep::Field(o.c)))),
        )
    }

    /// Narrows a reference to one array element, fixing the index now.
    pub fn reference_element(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let reference = self.reference_operand(o.b)?;
        let index = match self.read(register(o.c)?)? {
            Value::Integer(index) => *index,
            other => return Err(self.type_mismatch("an integer array index", other)),
        };
        self.write(
            register(o.a)?,
            Value::Reference(Arc::new(reference.narrowed(ReferenceStep::Element(index)))),
        )
    }

    /// Reads the current value through a reference.
    pub fn reference_read(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let reference = self.reference_operand(o.b)?;
        let value = self.read_reference(&reference)?;
        self.write(register(o.a)?, value)
    }

    /// Writes a value through a reference.
    pub fn reference_write(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let reference = self.reference_operand(o.a)?;
        let value = self.read(register(o.b)?)?.clone();
        self.write_reference(&reference, value)
    }

    /// Returns the current value of the referenced variable, field, or element.
    pub(in crate::vm) fn read_reference(
        &self,
        reference: &VariableReference,
    ) -> Result<Value, VmError> {
        let resolved = match &reference.root {
            ReferenceRoot::Cell(cell) => {
                let root = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                resolve_path(&root, &reference.path).cloned()
            }
            ReferenceRoot::Global(slot) => {
                let globals = self.global_slots();
                let root = usize::try_from(*slot)
                    .ok()
                    .and_then(|index| globals.get(index))
                    .and_then(Option::as_ref)
                    .ok_or_else(|| self.uninitialized_global(*slot))?;
                resolve_path(root, &reference.path).cloned()
            }
        };
        resolved.map_err(|failure| self.path_failure(failure))
    }

    /// Replaces the referenced variable, field, or element in the caller's storage.
    pub(in crate::vm) fn write_reference(
        &mut self,
        reference: &VariableReference,
        value: Value,
    ) -> Result<(), VmError> {
        match &reference.root {
            ReferenceRoot::Cell(cell) => {
                let mut root = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                replace_path(&mut root, &reference.path, value)
                    .map_err(|failure| self.path_failure(failure))
            }
            ReferenceRoot::Global(slot) => {
                let index = usize::try_from(*slot).map_err(|_| self.uninitialized_global(*slot))?;
                let mutable = self
                    .executable
                    .executable()
                    .globals
                    .get(index)
                    .is_some_and(|global| global.mutable);
                if !mutable {
                    return Err(self.aggregate_error(
                        format!("Global slot {slot} is immutable and cannot be written through a `var` parameter"),
                        "Recompile the program and report this internal bytecode invariant failure.",
                    ));
                }
                let outcome = {
                    let mut globals = self.global_slots_mut();
                    globals
                        .get_mut(index)
                        .and_then(Option::as_mut)
                        .map(|root| replace_path(root, &reference.path, value))
                };
                match outcome {
                    None => Err(self.uninitialized_global(*slot)),
                    Some(Err(failure)) => Err(self.path_failure(failure)),
                    Some(Ok(())) => {
                        self.note_debug_global_store(index);
                        Ok(())
                    }
                }
            }
        }
    }

    fn reference_operand(&self, operand: u16) -> Result<Arc<VariableReference>, VmError> {
        match self.read(register(operand)?)? {
            Value::Reference(reference) => Ok(reference.clone()),
            other => Err(self.type_mismatch("reference", other)),
        }
    }

    fn uninitialized_global(&self, slot: u32) -> VmError {
        self.aggregate_error(
            format!("Global slot {slot} was read before initialization"),
            "Initialize every global before its first read.",
        )
    }

    fn path_failure(&self, failure: PathFailure) -> VmError {
        match failure {
            PathFailure::NotRecord(actual) => self.aggregate_error(
                format!("Expected record behind a `var` field reference, got {actual}"),
                "Recompile the program and report this internal VM invariant failure.",
            ),
            PathFailure::NotArray(actual) => self.aggregate_error(
                format!("Expected array behind a `var` element reference, got {actual}"),
                "Recompile the program and report this internal VM invariant failure.",
            ),
            PathFailure::MissingField(field) => self.aggregate_error(
                format!("Verified record field slot {field} is unavailable"),
                "Recompile the program and report this internal bytecode invariant failure.",
            ),
            PathFailure::NegativeIndex(index) => self.aggregate_error_code(
                RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                format!("Negative array index {index} in a `var` argument"),
                "Array indices must be non-negative integers (0-based).",
            ),
            PathFailure::IndexOutOfBounds { index, len } => self.aggregate_error_code(
                RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                format!(
                    "Array index {index} of a `var` argument is out of bounds (len {len})"
                ),
                "The array changed size while the `var` parameter was in use. Check the index before the call, and avoid resizing the array through another name during the call.",
            ),
        }
    }
}

fn element_index(index: i64, len: usize) -> Result<usize, PathFailure> {
    let position = usize::try_from(index).map_err(|_| PathFailure::NegativeIndex(index))?;
    if position < len {
        Ok(position)
    } else {
        Err(PathFailure::IndexOutOfBounds { index, len })
    }
}

fn resolve_path<'a>(root: &'a Value, path: &[ReferenceStep]) -> Result<&'a Value, PathFailure> {
    let mut current = root;
    for step in path {
        current = match (step, current) {
            (ReferenceStep::Field(field), Value::Record(record)) => record
                .body()
                .values
                .get(usize::from(*field))
                .ok_or(PathFailure::MissingField(*field))?,
            (ReferenceStep::Field(_), other) => {
                return Err(PathFailure::NotRecord(other.type_name()));
            }
            (ReferenceStep::Element(index), Value::Array(values)) => {
                &values[element_index(*index, values.len())?]
            }
            (ReferenceStep::Element(_), other) => {
                return Err(PathFailure::NotArray(other.type_name()));
            }
        };
    }
    Ok(current)
}

fn replace_path(
    current: &mut Value,
    path: &[ReferenceStep],
    replacement: Value,
) -> Result<(), PathFailure> {
    let Some((step, tail)) = path.split_first() else {
        *current = replacement;
        return Ok(());
    };
    match (step, current) {
        (ReferenceStep::Field(field), Value::Record(record)) => {
            let child = record
                .values_mut()
                .get_mut(usize::from(*field))
                .ok_or(PathFailure::MissingField(*field))?;
            replace_path(child, tail, replacement)
        }
        (ReferenceStep::Field(_), other) => Err(PathFailure::NotRecord(other.type_name())),
        (ReferenceStep::Element(index), Value::Array(values)) => {
            let position = element_index(*index, values.len())?;
            replace_path(&mut values[position], tail, replacement)
        }
        (ReferenceStep::Element(_), other) => Err(PathFailure::NotArray(other.type_name())),
    }
}
