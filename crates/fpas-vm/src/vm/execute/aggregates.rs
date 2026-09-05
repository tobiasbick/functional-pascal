//! Dense global and positional aggregate operations.

mod arrays;

use fpas_bytecode::{
    AbcOperands, AbxOperands, SharedEnum, SharedRecord, Value, managed_value_buffer,
};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::execute::scalar::register;
use super::super::value_ops::{self, BinaryOperation};
use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

impl Worker {
    pub fn load_global(&mut self, o: AbxOperands) -> Result<(), VmError> {
        let index = usize::try_from(o.bx).map_err(|_| self.bad_slot("global", o.bx))?;
        let value = self
            .global_slots()
            .get(index)
            .cloned()
            .flatten()
            .ok_or_else(|| {
                self.aggregate_error(
                    format!("Global slot {} was read before initialization", o.bx),
                    "Initialize every global before its first read.",
                )
            })?;
        self.write(register(o.a)?, value)
    }

    pub fn store_global(&mut self, o: AbxOperands) -> Result<(), VmError> {
        let value = self.read(register(o.a)?)?.clone();
        let index = usize::try_from(o.bx).map_err(|_| self.bad_slot("global", o.bx))?;
        let mutable = self
            .executable
            .executable()
            .globals
            .get(index)
            .ok_or_else(|| self.bad_slot("global", o.bx))?
            .mutable;
        enum StoreOutcome {
            Stored,
            Immutable,
            Missing,
        }
        let outcome = {
            let mut globals = self.global_slots_mut();
            if !mutable && globals.get(index).is_some_and(Option::is_some) {
                StoreOutcome::Immutable
            } else if let Some(slot) = globals.get_mut(index) {
                *slot = Some(value);
                StoreOutcome::Stored
            } else {
                StoreOutcome::Missing
            }
        };
        match outcome {
            StoreOutcome::Stored => {}
            StoreOutcome::Immutable => {
                return Err(self.aggregate_error(
                    format!("Immutable global slot {} was assigned more than once", o.bx),
                    "Assign immutable globals only during initialization.",
                ));
            }
            StoreOutcome::Missing => return Err(self.bad_slot("global", o.bx)),
        }
        self.note_debug_global_store(index);
        Ok(())
    }

    pub fn make_dictionary(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let count = usize::from(o.c)
            .checked_mul(2)
            .ok_or_else(|| self.bad_slot("dictionary window", u32::from(o.c)))?;
        let values = self.window(o.b, count)?;
        let pairs = values
            .as_chunks::<2>()
            .0
            .iter()
            .map(|[key, value]| (key.clone(), value.clone()))
            .collect();
        self.write(register(o.a)?, Value::dict(pairs))
    }

    pub fn index_get(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let collection = self.read(register(o.b)?)?.clone();
        let index = self.read(register(o.c)?)?.clone();
        let value = value_ops::index(&collection, &index)
            .map_err(|error| self.aggregate_error_code(error.code, error.message, error.hint))?;
        self.write(register(o.a)?, value)
    }

    /// Consumes and updates the collection held in the destination register.
    pub fn index_set(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let destination = register(o.a)?;
        let index = self.read(register(o.b)?)?.clone();
        let value = self.read(register(o.c)?)?.clone();
        let array_index = match self.read(destination)? {
            Value::Array(values) => {
                let index = self.array_index(&index)?;
                let length = values.len();
                if index >= length {
                    return Err(self.aggregate_error_code(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Array index {index} out of bounds (len {length})"),
                        "Check index bounds before array assignment.",
                    ));
                }
                Some(index)
            }
            Value::Dict(_) => None,
            other => return Err(self.type_mismatch("array or dictionary", other)),
        };
        let collection = self.take(destination)?;
        let updated = match collection {
            Value::Array(mut values) => {
                let Some(index) = array_index else {
                    return Err(diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "Validated IndexSet array lost its index",
                    ));
                };
                values[index] = value;
                Value::Array(values)
            }
            Value::Dict(mut pairs) => {
                let key = index;
                if let Some((_, existing)) =
                    pairs.iter_mut().find(|(candidate, _)| *candidate == key)
                {
                    *existing = value;
                } else {
                    pairs.push((key, value));
                }
                Value::Dict(pairs)
            }
            _ => {
                return Err(diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Validated IndexSet destination changed type before commit",
                ));
            }
        };
        self.write(destination, updated)
    }

    pub fn contains(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let needle = self.read(register(o.b)?)?.clone();
        let aggregate = self.read(register(o.c)?)?.clone();
        let found = value_ops::binary(BinaryOperation::In, &needle, &aggregate)
            .map_err(|error| self.aggregate_error_code(error.code, error.message, error.hint))?;
        self.write(register(o.a)?, found)
    }

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

    pub fn update_record(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let mut record = self.read(register(o.a)?)?.clone();
        if !matches!(record, Value::Record(_)) {
            return Err(self.type_mismatch("record", &record));
        }
        let overrides = self.window(o.b, usize::from(o.c) * 2)?;
        for [field, value] in overrides.as_chunks::<2>().0 {
            let Value::Integer(field) = field else {
                return Err(self.type_mismatch("integer record field slot", field));
            };
            let field =
                usize::try_from(*field).map_err(|_| self.bad_slot("record field", u32::MAX))?;
            match &mut record {
                Value::Record(record) => {
                    *record.values_mut().get_mut(field).ok_or_else(|| {
                        self.bad_slot("record field", u32::try_from(field).unwrap_or(u32::MAX))
                    })? = value.clone()
                }
                _ => return Err(self.type_mismatch("record", &record)),
            }
        }
        self.write(register(o.a)?, record)
    }

    pub fn make_enum(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let variant = self
            .layouts
            .enum_variants
            .get(usize::from(o.b))
            .cloned()
            .ok_or_else(|| self.bad_slot("enum variant", u32::from(o.b)))?;
        let values = self.window(o.c, variant.fields.len())?;
        self.write(
            register(o.a)?,
            Value::Enum(SharedEnum::new(variant, values)),
        )
    }

    pub fn test_variant(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let variant = self
            .layouts
            .enum_variants
            .get(usize::from(o.c))
            .ok_or_else(|| self.bad_slot("enum variant", u32::from(o.c)))?;
        let matches = match self.read(register(o.b)?)? {
            Value::Enum(value) => {
                value.body().layout.enumeration == variant.enumeration
                    && value.body().layout.variant_id == variant.variant_id
            }
            other => return Err(self.type_mismatch("enum", other)),
        };
        self.write(register(o.a)?, Value::Boolean(matches))
    }

    pub fn load_enum_field(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let value = match self.read(register(o.b)?)? {
            Value::Enum(value) => value.body().values.get(usize::from(o.c)).cloned(),
            other => return Err(self.type_mismatch("enum", other)),
        }
        .ok_or_else(|| self.bad_slot("enum field", u32::from(o.c)))?;
        self.write(register(o.a)?, value)
    }

    pub fn wrap(&mut self, o: AbcOperands, constructor: fn(Value) -> Value) -> Result<(), VmError> {
        let value = self.read(register(o.b)?)?.clone();
        self.write(register(o.a)?, constructor(value))
    }
    pub fn none(&mut self, o: AbcOperands) -> Result<(), VmError> {
        self.write(register(o.a)?, Value::OptionNone)
    }
    pub fn test_ok(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let yes = matches!(self.read(register(o.b)?)?, Value::ResultOk(_));
        self.write(register(o.a)?, Value::Boolean(yes))
    }
    pub fn test_some(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let yes = matches!(self.read(register(o.b)?)?, Value::OptionSome(_));
        self.write(register(o.a)?, Value::Boolean(yes))
    }
    pub fn unwrap(&mut self, o: AbcOperands, kind: &str) -> Result<(), VmError> {
        let value = match (kind, self.read(register(o.b)?)?) {
            ("Ok", Value::ResultOk(v))
            | ("Error", Value::ResultError(v))
            | ("Some", Value::OptionSome(v)) => Some((**v).clone()),
            _ => None,
        }
        .ok_or_else(|| {
            self.aggregate_error(
                format!("Cannot unwrap {kind} from this value"),
                "Test the Result or Option variant before unwrapping.",
            )
        })?;
        self.write(register(o.a)?, value)
    }

    /// Clones one verified contiguous register window.
    pub(super) fn window(&self, base: u16, count: usize) -> Result<Vec<Value>, VmError> {
        let mut values = managed_value_buffer(count);
        for offset in 0..count {
            let slot = usize::from(base)
                .checked_add(offset)
                .ok_or_else(|| self.bad_slot("register window", u32::from(base)))?;
            let value = self
                .registers
                .get(..self.active_register_count)
                .and_then(|registers| registers.get(self.base + slot))
                .cloned()
                .ok_or_else(|| {
                    self.bad_slot("register window", u32::try_from(slot).unwrap_or(u32::MAX))
                })?;
            values.push(value);
        }
        Ok(values)
    }
    fn bad_slot(&self, kind: &str, slot: u32) -> VmError {
        self.aggregate_error(
            format!("Verified {kind} slot {slot} is unavailable"),
            "Recompile the program and report this internal bytecode invariant failure.",
        )
    }
    /// Creates an aggregate runtime diagnostic at the current instruction.
    pub(super) fn aggregate_error(
        &self,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> VmError {
        self.aggregate_error_code(RUNTIME_VM_OPERAND_TYPE_MISMATCH, message, hint)
    }
    /// Creates an aggregate runtime diagnostic with an explicit stable code.
    pub(super) fn aggregate_error_code(
        &self,
        code: DiagnosticCode,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            code,
            message,
            hint,
        )
    }
    /// Creates the standard aggregate operand type-mismatch diagnostic.
    pub(super) fn type_mismatch(&self, expected: &str, actual: &Value) -> VmError {
        self.aggregate_error(
            format!("Expected {expected}, got {}", actual.type_name()),
            format!("Use {expected} operands for this operation."),
        )
    }
    /// Converts an FPAS array key to a checked host index.
    pub(super) fn array_index(&self, key: &Value) -> Result<usize, VmError> {
        match key {
            Value::Integer(index) if *index >= 0 => usize::try_from(*index).map_err(|_| {
                self.aggregate_error_code(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("Array index {index} cannot be represented on this host"),
                    "Use a smaller non-negative array index.",
                )
            }),
            Value::Integer(index) => Err(self.aggregate_error_code(
                RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                format!("Negative array index {index}"),
                "Array indices must be non-negative integers (0-based).",
            )),
            other => Err(self.type_mismatch("an integer array index", other)),
        }
    }
}
