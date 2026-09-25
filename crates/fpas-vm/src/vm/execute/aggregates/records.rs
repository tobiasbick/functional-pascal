//! Record construction, field access, and in-place record updates.
//! Documentation: `docs/pascal/language/types/records.md`.

use super::*;

impl Worker {
    pub fn make_record(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let layout = self
            .layouts
            .records
            .get(usize::from(o.b))
            .cloned()
            .ok_or_else(|| self.bad_slot("record layout", u32::from(o.b)))?;
        let values = self.window(o.c, layout.fields.len())?;
        self.write(
            register(o.a)?,
            Value::Record(SharedRecord::new(layout, values)),
        )
    }

    pub fn load_field(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let value = match self.read(register(o.b)?)? {
            Value::Record(record) => record.body().values.get(usize::from(o.c)).cloned(),
            other => return Err(self.type_mismatch("record", other)),
        }
        .ok_or_else(|| self.bad_slot("record field", u32::from(o.c)))?;
        self.write(register(o.a)?, value)
    }

    /// Updates a validated field, reusing uniquely owned record storage.
    pub fn store_field(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let value = self.read(register(o.c)?)?.clone();
        let destination = register(o.a)?;
        let field = usize::from(o.b);
        match self.read(destination)? {
            Value::Record(record) if field < record.body().values.len() => {}
            Value::Record(_) => return Err(self.bad_slot("record field", u32::from(o.b))),
            other => return Err(self.type_mismatch("record", other)),
        }
        let Value::Record(mut record) = self.take(destination)? else {
            return Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Validated StoreField destination changed type before commit",
            ));
        };
        record.values_mut()[field] = value;
        self.write(destination, Value::Record(record))
    }

    /// Apply positional overrides to the record in register A.
    ///
    /// Overrides are validated first; the record then leaves its register so a uniquely owned
    /// body is updated in place instead of being copied.
    pub fn update_record(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let destination = register(o.a)?;
        let field_count = match self.read(destination)? {
            Value::Record(record) => record.body().values.len(),
            other => return Err(self.type_mismatch("record", other)),
        };
        let start = self.base + usize::from(o.b);
        let end = start + usize::from(o.c) * 2;
        let Some(overrides) = self.registers.get(start..end) else {
            return Err(self.bad_slot("register window", u32::from(o.b)));
        };
        for pair in overrides.chunks_exact(2) {
            let Value::Integer(field) = &pair[0] else {
                return Err(self.type_mismatch("integer record field slot", &pair[0]));
            };
            match usize::try_from(*field) {
                Ok(field) if field < field_count => {}
                _ => {
                    return Err(
                        self.bad_slot("record field", u32::try_from(*field).unwrap_or(u32::MAX))
                    );
                }
            }
        }
        let destination_index = self.base + usize::from(o.a);
        // A destination inside the override window must stay readable, so keep a shared copy.
        let mut record = if (start..end).contains(&destination_index) {
            self.read(destination)?.clone()
        } else {
            self.take(destination)?
        };
        let Value::Record(body) = &mut record else {
            return Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Validated UpdateRecord destination changed type before commit",
            ));
        };
        let values = body.values_mut();
        for pair in self.registers[start..end].chunks_exact(2) {
            // Field slots were validated above.
            if let Value::Integer(field) = pair[0] {
                values[field as usize] = pair[1].clone();
            }
        }
        self.write(destination, record)
    }
}
