//! Dense global and positional aggregate operations.

use fpas_bytecode::{AbcOperands, AbxOperands, SharedEnum, SharedRecord, Value};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::execute::scalar::register;
use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

impl Worker {
    pub fn load_global(&mut self, o: AbxOperands) -> Result<(), VmError> {
        let index = usize::try_from(o.bx).map_err(|_| self.bad_slot("global", o.bx))?;
        let value = self
            .globals
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
        let mut globals = self
            .globals
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !mutable && globals.get(index).is_some_and(Option::is_some) {
            return Err(self.aggregate_error(
                format!("Immutable global slot {} was assigned more than once", o.bx),
                "Assign immutable globals only during initialization.",
            ));
        }
        let Some(slot) = globals.get_mut(index) else {
            return Err(self.bad_slot("global", o.bx));
        };
        *slot = Some(value);
        Ok(())
    }

    pub fn make_array(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let values = self.window(o.b, usize::from(o.c))?;
        self.write(register(o.a)?, Value::Array(values.into()))
    }

    pub fn make_dictionary(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let count = usize::from(o.c)
            .checked_mul(2)
            .ok_or_else(|| self.bad_slot("dictionary window", u32::from(o.c)))?;
        let values = self.window(o.b, count)?;
        let pairs = values
            .chunks_exact(2)
            .map(|pair| (pair[0].clone(), pair[1].clone()))
            .collect();
        self.write(register(o.a)?, Value::dict(pairs))
    }

    pub fn index_get(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let collection = self.read(register(o.b)?)?;
        let index = self.read(register(o.c)?)?;
        let value = match (collection, index) {
            (Value::Array(values), key) => {
                let index = self.array_index(key)?;
                values.get(index).cloned().ok_or_else(|| {
                    self.aggregate_error_code(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Array index {index} out of bounds (len {})", values.len()),
                        "Check index bounds before array access.",
                    )
                })?
            }
            (Value::Dict(pairs), key) => pairs
                .iter()
                .find(|(candidate, _)| candidate == key)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| {
                    self.aggregate_error_code(
                        RUNTIME_DICT_KEY_NOT_FOUND,
                        format!("Key `{key}` not found in dict"),
                        "Use Std.Dict.ContainsKey to check before access.",
                    )
                })?,
            (Value::Str(text), key) => {
                let index = self.array_index(key)?;
                if index >= text.len() {
                    return Err(self.string_index_error(index));
                }
                let character = text
                    .chars()
                    .nth(index)
                    .ok_or_else(|| self.string_index_error(index))?;
                Value::Str(character.to_string().into())
            }
            _ => {
                return Err(self.type_mismatch("an array, dictionary, or string", collection));
            }
        };
        self.write(register(o.a)?, value)
    }

    pub fn index_set(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let index = self.read(register(o.b)?)?.clone();
        let value = self.read(register(o.c)?)?.clone();
        let collection = self.read(register(o.a)?)?.clone();
        let updated = match (collection, index) {
            (Value::Array(mut values), Value::Integer(index)) => {
                let key = Value::Integer(index);
                let index = self.array_index(&key)?;
                let length = values.len();
                *values.get_mut(index).ok_or_else(|| {
                    self.aggregate_error_code(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Array index {index} out of bounds (len {length})"),
                        "Check index bounds before array assignment.",
                    )
                })? = value;
                Value::Array(values)
            }
            (Value::Dict(mut pairs), key) => {
                if let Some((_, existing)) =
                    pairs.iter_mut().find(|(candidate, _)| *candidate == key)
                {
                    *existing = value;
                } else {
                    pairs.push((key, value));
                }
                Value::Dict(pairs)
            }
            (other, _) => return Err(self.type_mismatch("array or dictionary", &other)),
        };
        self.write(register(o.a)?, updated)
    }

    pub fn contains(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let needle = self.read(register(o.b)?)?;
        let aggregate = self.read(register(o.c)?)?;
        let found = match aggregate {
            Value::Array(values) => values.iter().any(|value| value == needle),
            Value::Dict(pairs) => pairs.iter().any(|(key, _)| key == needle),
            Value::Str(text) => self.string_contains(text, needle)?,
            other => return Err(self.type_mismatch("array, dictionary, or string", other)),
        };
        self.write(register(o.a)?, Value::Boolean(found))
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

    pub fn store_field(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let value = self.read(register(o.c)?)?.clone();
        let record = match self.read(register(o.a)?)?.clone() {
            Value::Record(mut record) => {
                *record
                    .values_mut()
                    .get_mut(usize::from(o.b))
                    .ok_or_else(|| self.bad_slot("record field", u32::from(o.b)))? = value;
                Value::Record(record)
            }
            other => return Err(self.type_mismatch("record", &other)),
        };
        self.write(register(o.a)?, record)
    }

    pub fn update_record(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let mut record = self.read(register(o.a)?)?.clone();
        if !matches!(record, Value::Record(_)) {
            return Err(self.type_mismatch("record", &record));
        }
        let overrides = self.window(o.b, usize::from(o.c) * 2)?;
        for pair in overrides.chunks_exact(2) {
            let Value::Integer(field) = pair[0] else {
                return Err(self.type_mismatch("integer record field slot", &pair[0]));
            };
            let field =
                usize::try_from(field).map_err(|_| self.bad_slot("record field", u32::MAX))?;
            match &mut record {
                Value::Record(record) => {
                    *record.values_mut().get_mut(field).ok_or_else(|| {
                        self.bad_slot("record field", u32::try_from(field).unwrap_or(u32::MAX))
                    })? = pair[1].clone()
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

    pub fn wrap(
        &mut self,
        o: AbcOperands,
        constructor: fn(Box<Value>) -> Value,
    ) -> Result<(), VmError> {
        let value = self.read(register(o.b)?)?.clone();
        self.write(register(o.a)?, constructor(Box::new(value)))
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

    fn window(&self, base: u16, count: usize) -> Result<Vec<Value>, VmError> {
        (0..count)
            .map(|offset| {
                let slot = usize::from(base)
                    .checked_add(offset)
                    .ok_or_else(|| self.bad_slot("register window", u32::from(base)))?;
                self.registers
                    .get(self.base + slot)
                    .cloned()
                    .ok_or_else(|| {
                        self.bad_slot("register window", u32::try_from(slot).unwrap_or(u32::MAX))
                    })
            })
            .collect()
    }
    fn bad_slot(&self, kind: &str, slot: u32) -> VmError {
        self.aggregate_error(
            format!("Verified {kind} slot {slot} is unavailable"),
            "Recompile the program and report this internal bytecode invariant failure.",
        )
    }
    fn aggregate_error(&self, message: impl Into<String>, hint: impl Into<String>) -> VmError {
        self.aggregate_error_code(RUNTIME_VM_OPERAND_TYPE_MISMATCH, message, hint)
    }
    fn aggregate_error_code(
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
    fn type_mismatch(&self, expected: &str, actual: &Value) -> VmError {
        self.aggregate_error(
            format!("Expected {expected}, got {}", actual.type_name()),
            format!("Use {expected} operands for this operation."),
        )
    }
    fn array_index(&self, key: &Value) -> Result<usize, VmError> {
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
    fn string_index_error(&self, index: usize) -> VmError {
        self.aggregate_error_code(
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("String index {index} out of bounds"),
            "Check the index is in the range 0 .. Length(S) - 1.",
        )
    }
    fn string_contains(&self, text: &str, needle: &Value) -> Result<bool, VmError> {
        let Value::Str(value) = needle else {
            return Err(self.aggregate_error(
                "String membership requires a string value",
                "Use `Substring in Text` for string membership.",
            ));
        };
        let mut characters = value.chars();
        Ok(match (characters.next(), characters.next()) {
            (Some(character), None) => text.chars().any(|candidate| candidate == character),
            _ => text.contains(value.as_ref()),
        })
    }
}
