//! Preserve executable-global watch identities for cell and selected-reference writes.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use crate::vm::worker::Worker;
use fpas_bytecode::Value;
use std::sync::Arc;

impl Worker {
    /// Notify the existing global watch path after a successful authorized write.
    pub(in crate::vm) fn note_debug_storage_store(&mut self, storage: &Value) {
        if !self.debug_tasks {
            return;
        }
        let index = self.globals.read().ok().and_then(|globals| {
            globals.iter().position(|slot| match (storage, slot) {
                (Value::Cell(written), Some(Value::Cell(global))) => Arc::ptr_eq(written, global),
                (Value::Reference(written), Some(Value::Cell(global))) => {
                    written.matches_cell(global)
                }
                _ => false,
            })
        });
        if let Some(index) = index {
            self.note_debug_global_store(index);
        }
    }
}
