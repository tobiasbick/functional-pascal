//! First-reference deterministic executable string interning.

use std::collections::HashMap;

use fpas_bytecode::{StringId, StringTable};

use crate::LinkError;

#[derive(Default)]
pub(super) struct StringInterner {
    values: Vec<String>,
    indices: HashMap<String, StringId>,
}

impl StringInterner {
    pub(super) fn intern(&mut self, value: &str) -> Result<StringId, LinkError> {
        if let Some(id) = self.indices.get(value) {
            return Ok(*id);
        }
        let id = StringId::try_from_index(self.values.len())
            .map_err(|_| LinkError::Overflow("string IDs"))?;
        self.values.push(value.to_string());
        self.indices.insert(value.to_string(), id);
        Ok(id)
    }

    pub(super) fn finish(self) -> StringTable {
        StringTable::new(self.values)
    }
}
