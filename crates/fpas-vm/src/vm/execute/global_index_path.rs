//! In-place indexed updates of global aggregate snapshots.

use fpas_bytecode::{AbcOperands, Value};
use fpas_diagnostics::codes::RUNTIME_DICT_KEY_NOT_FOUND;

use super::super::VmError;
use super::super::execute::scalar::register;
use super::super::worker::Worker;

enum ResolvedIndex {
    Array(usize),
    Dictionary {
        position: usize,
        missing_key: Option<Value>,
    },
}

impl Worker {
    /// Replaces one value below a global aggregate snapshot without rebuilding its path.
    pub fn store_global_index_path(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let index_count = usize::from(o.auxiliary);
        let window = self.window(o.c, index_count.saturating_add(1))?;
        let Some((value, indexes)) = window.split_last() else {
            return Err(self.aggregate_error(
                "Global index path window has no replacement value",
                "Recompile the program and report this internal bytecode invariant failure.",
            ));
        };
        let root_register = register(o.a)?;
        let path = self.resolve_global_index_path(self.read(root_register)?, indexes)?;
        let global_index = usize::from(o.b);
        let mutable = self
            .executable
            .executable()
            .globals
            .get(global_index)
            .ok_or_else(|| self.bad_global_path_slot(o.b))?
            .mutable;
        if !mutable {
            return Err(self.aggregate_error(
                format!("Immutable global slot {} was assigned more than once", o.b),
                "Assign immutable globals only during initialization.",
            ));
        }

        let mut root = self.take(root_register)?;
        enum StoreOutcome {
            Stored,
            Missing,
            Uninitialized,
            PathMismatch,
        }
        let outcome = {
            let mut globals = self.global_slots_mut();
            match globals.get_mut(global_index) {
                None => StoreOutcome::Missing,
                Some(slot) => match slot.take() {
                    None => StoreOutcome::Uninitialized,
                    Some(current) => {
                        drop(current);
                        if !replace_resolved_path(&mut root, &path, value.clone()) {
                            *slot = Some(root);
                            StoreOutcome::PathMismatch
                        } else {
                            *slot = Some(root);
                            StoreOutcome::Stored
                        }
                    }
                },
            }
        };
        match outcome {
            StoreOutcome::Stored => {}
            StoreOutcome::Missing => return Err(self.bad_global_path_slot(o.b)),
            StoreOutcome::Uninitialized => {
                return Err(self.aggregate_error(
                    format!("Global slot {} was read before initialization", o.b),
                    "Initialize every global before its first read.",
                ));
            }
            StoreOutcome::PathMismatch => {
                return Err(self.aggregate_error(
                    "Resolved global index path no longer matches its aggregate",
                    "Recompile the program and report this internal VM invariant failure.",
                ));
            }
        }
        self.note_debug_global_store(global_index);
        Ok(())
    }

    fn resolve_global_index_path(
        &self,
        root: &Value,
        indexes: &[Value],
    ) -> Result<Vec<ResolvedIndex>, VmError> {
        let mut current = root;
        let mut resolved = Vec::with_capacity(indexes.len());
        for (offset, key) in indexes.iter().enumerate() {
            let is_leaf = offset + 1 == indexes.len();
            match current {
                Value::Array(values) => {
                    let index = self.array_index(key)?;
                    current = values.get(index).ok_or_else(|| {
                        self.aggregate_error_code(
                            fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                            format!("Array index {index} out of bounds (len {})", values.len()),
                            "Check index bounds before array assignment.",
                        )
                    })?;
                    resolved.push(ResolvedIndex::Array(index));
                }
                Value::Dict(pairs) => {
                    if let Some(position) = pairs.iter().position(|(candidate, _)| candidate == key)
                    {
                        current = &pairs[position].1;
                        resolved.push(ResolvedIndex::Dictionary {
                            position,
                            missing_key: None,
                        });
                    } else if is_leaf {
                        resolved.push(ResolvedIndex::Dictionary {
                            position: pairs.len(),
                            missing_key: Some(key.clone()),
                        });
                    } else {
                        return Err(self.aggregate_error_code(
                            RUNTIME_DICT_KEY_NOT_FOUND,
                            format!("Key `{key}` not found in dict"),
                            "Use Std.Dict.ContainsKey to check before access.",
                        ));
                    }
                }
                other => return Err(self.type_mismatch("array or dictionary", other)),
            }
        }
        Ok(resolved)
    }

    fn bad_global_path_slot(&self, slot: u16) -> VmError {
        self.aggregate_error(
            format!("Verified global slot {slot} is unavailable"),
            "Recompile the program and report this internal bytecode invariant failure.",
        )
    }
}

fn replace_resolved_path(current: &mut Value, path: &[ResolvedIndex], replacement: Value) -> bool {
    let Some((step, tail)) = path.split_first() else {
        *current = replacement;
        return true;
    };
    match (current, step) {
        (Value::Array(values), ResolvedIndex::Array(index)) => values
            .get_mut(*index)
            .is_some_and(|child| replace_resolved_path(child, tail, replacement)),
        (
            Value::Dict(pairs),
            ResolvedIndex::Dictionary {
                position,
                missing_key,
            },
        ) => {
            if let Some(key) = missing_key {
                if !tail.is_empty() {
                    return false;
                }
                pairs.push((key.clone(), replacement));
                true
            } else if let Some((_, child)) = pairs.get_mut(*position) {
                replace_resolved_path(child, tail, replacement)
            } else {
                false
            }
        }
        _ => false,
    }
}
