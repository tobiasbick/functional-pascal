//! Aggregate value execution: arrays, dicts, records, and local array mutation.
//!
//! **Documentation:** `docs/pascal/02-basics.md`, `docs/pascal/std/array.md` (from the repository root).

mod array_locals;
mod indexing;
mod records;
mod refs;

use super::super::Worker;
use super::super::diagnostics::VmError;
use fpas_bytecode::{Op, SourceLocation, Value};

impl Worker {
    pub(super) fn try_exec_aggregates(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::MakeArray(count) => {
                self.exec_make_array(count)?;
                Ok(true)
            }
            Op::MakeDict(pair_count) => {
                self.exec_make_dict(pair_count)?;
                Ok(true)
            }
            Op::IndexGet => {
                self.exec_index_get(line)?;
                Ok(true)
            }
            Op::IndexSet => {
                self.exec_index_set(line)?;
                Ok(true)
            }
            Op::MakeRecord(type_idx, field_count) => {
                self.exec_make_record(type_idx, field_count, line)?;
                Ok(true)
            }
            Op::MakeRef(type_idx) => {
                self.exec_make_ref(type_idx, line)?;
                Ok(true)
            }
            Op::FieldGet(name_idx) => {
                self.exec_field_get(name_idx, line)?;
                Ok(true)
            }
            Op::FieldSet(name_idx) => {
                self.exec_field_set(name_idx, line)?;
                Ok(true)
            }
            Op::ArrayPushLocal(depth, slot) => {
                self.exec_array_push_local(depth, slot, line)?;
                Ok(true)
            }
            Op::ArrayPopLocal(depth, slot) => {
                self.exec_array_pop_local(depth, slot, line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn drain_values(&mut self, count: usize) -> Vec<Value> {
        let start = self.stack.len() - count;
        self.stack.drain(start..).collect()
    }

    fn exec_make_array(&mut self, count: u16) -> Result<(), VmError> {
        let elements = self.drain_values(count as usize);
        self.push(Value::Array(elements))?;
        Ok(())
    }

    fn exec_make_dict(&mut self, pair_count: u16) -> Result<(), VmError> {
        let items = self.drain_values(pair_count as usize * 2);
        let pairs = items
            .chunks(2)
            .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
            .collect();
        self.push(Value::Dict(pairs))?;
        Ok(())
    }
}
