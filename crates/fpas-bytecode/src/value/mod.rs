mod aggregate;
mod array;
mod display;
mod equal;
mod function;
mod managed_heap;
mod string;

pub use aggregate::{
    EnumValue, RecordValue, RuntimeEnumLayout, RuntimeRecordLayout, SharedDict, SharedEnum,
    SharedRecord,
};
pub use array::SharedArray;
use equal::values_equal;
pub use function::{FunctionValue, SharedFunction};
pub use managed_heap::managed_value_buffer;
pub use string::SharedStr;

/// Runtime value in the VM.
#[derive(Debug, Clone)]
pub enum Value {
    /// Signed integer value.
    Integer(i64),
    /// Double-precision floating-point value.
    Real(f64),
    /// Boolean value.
    Boolean(bool),
    /// Shared UTF-8 string value.
    Str(SharedStr),
    /// Enum variant with optional associated data.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    Enum(SharedEnum),
    /// Ordered collection.
    Array(SharedArray),
    /// Key-value collection (ordered by insertion).
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    Dict(SharedDict),
    /// Record whose values follow its executable-shared field layout.
    Record(SharedRecord),
    /// Unit / void — result of procedures, statements.
    Unit,
    /// Result::Ok wrapped value.
    ResultOk(Box<Value>),
    /// Result::Error wrapped value.
    ResultError(Box<Value>),
    /// Option::Some wrapped value.
    OptionSome(Box<Value>),
    /// Option::None sentinel.
    OptionNone,
    /// First-class function value (named or anonymous).
    ///
    /// For closures, `captures` holds values or [`Value::Cell`] handles captured from
    /// enclosing scopes. `task_bound` is true when any capture is a mutable cell.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    Function(SharedFunction),
    /// Shared mutable capture cell. Cloning shares the cell; the inner value is updated in place.
    ///
    /// Uses [`std::sync::Arc`] / [`std::sync::Mutex`] so [`Value`] stays `Send` + `Sync` for the
    /// multi-threaded VM. Mutable captures still mark closures as `task_bound`.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    Cell(std::sync::Arc<std::sync::Mutex<Value>>),
    /// Task handle (runtime id).
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Task(u64),
    /// Opaque host-resource handle that FPAS code can only pass back to its owning intrinsic.
    OpaqueHandle(u64),
}

impl Value {
    /// Create an ordered dictionary value with copy-on-write storage.
    pub fn dict(pairs: Vec<(Value, Value)>) -> Self {
        Self::Dict(pairs.into())
    }

    /// Create a non-task-bound first-class function value.
    pub fn function(function: crate::FunctionId, name: String, captures: Vec<Value>) -> Self {
        Self::Function(SharedFunction::unbound(function, name, captures))
    }

    /// Create a first-class method value bound to one immutable receiver snapshot.
    pub fn bound_function(function: crate::FunctionId, name: String, receiver: Value) -> Self {
        Self::Function(SharedFunction::bound(function, name, receiver))
    }

    /// Create a task-bound function owned by one runtime task.
    ///
    /// The owner token is runtime-only and is never stored in program artifacts.
    pub fn task_owned_function(
        function: crate::FunctionId,
        name: String,
        captures: Vec<Value>,
        owner_task: u64,
    ) -> Self {
        Self::Function(SharedFunction::task_owned(
            function, name, captures, owner_task,
        ))
    }

    /// Return the runtime type category name for diagnostics.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Real(_) => "real",
            Value::Boolean(_) => "boolean",
            Value::Str(_) => "string",
            Value::Enum(_) => "enum",
            Value::Array(_) => "array",
            Value::Dict(_) => "dict",
            Value::Record(_) => "record",
            Value::Unit => "unit",
            Value::ResultOk(_) => "Result.Ok",
            Value::ResultError(_) => "Result.Error",
            Value::OptionSome(_) => "Option.Some",
            Value::OptionNone => "Option.None",
            Value::Function(_) => "function",
            Value::Cell(_) => "cell",
            Value::Task(_) => "task",
            Value::OpaqueHandle(_) => "opaque handle",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        values_equal(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(type_name: &str, fields: &[&str], values: Vec<Value>) -> Value {
        Value::Record(SharedRecord::new(
            std::sync::Arc::new(RuntimeRecordLayout {
                record: crate::RecordTypeId::new(0),
                type_name: type_name.to_string(),
                fields: fields.iter().map(|field| (*field).to_string()).collect(),
            }),
            values,
        ))
    }

    fn enumeration(type_name: &str, variant: &str, fields: Vec<Value>) -> Value {
        Value::Enum(SharedEnum::new(
            std::sync::Arc::new(RuntimeEnumLayout {
                enumeration: crate::EnumTypeId::new(0),
                variant_id: crate::EnumVariantId::new(0),
                type_name: type_name.to_string(),
                variant: variant.to_string(),
                fields: (0..fields.len())
                    .map(|index| {
                        if fields.len() == 1 {
                            "value".to_string()
                        } else {
                            format!("field{index}")
                        }
                    })
                    .collect(),
            }),
            fields,
        ))
    }

    #[test]
    fn type_name_reports_runtime_categories() {
        assert_eq!(Value::Integer(1).type_name(), "integer");
        assert_eq!(Value::dict(Vec::new()).type_name(), "dict");
        assert_eq!(Value::Task(9).type_name(), "task");
        assert_eq!(Value::OpaqueHandle(9).type_name(), "opaque handle");
    }

    #[test]
    fn display_formats_dict_with_braces() {
        let value = Value::dict(vec![
            (Value::Str("k".into()), Value::Integer(1)),
            (Value::Str("x".into()), Value::Boolean(true)),
        ]);
        assert_eq!(value.to_string(), "{k: 1, x: true}");
    }

    #[test]
    fn display_formats_self_referential_cell_opaquely() {
        let cell = std::sync::Arc::new(std::sync::Mutex::new(Value::Unit));
        *cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Value::Cell(cell.clone());

        assert_eq!(Value::Cell(cell).to_string(), "<cell>");
    }

    #[test]
    fn partial_eq_treats_nan_with_same_bits_as_equal() {
        let nan = Value::Real(f64::from_bits(0x7FF8_0000_0000_0001));
        assert_eq!(nan, nan.clone());
    }

    #[test]
    fn partial_eq_distinguishes_nan_payloads() {
        let a = Value::Real(f64::from_bits(0x7FF8_0000_0000_0001));
        let b = Value::Real(f64::from_bits(0x7FF8_0000_0000_0002));
        assert_ne!(a, b);
    }

    #[test]
    fn partial_eq_compares_nested_arrays() {
        let left = Value::Array(vec![Value::Array(vec![Value::Integer(1)].into())].into());
        let right = Value::Array(vec![Value::Array(vec![Value::Integer(1)].into())].into());
        assert_eq!(left, right);
    }

    #[test]
    fn partial_eq_compares_enum_fields_structurally() {
        let left = enumeration(
            "Result",
            "Ok",
            vec![Value::Array(vec![Value::Integer(1)].into())],
        );
        let right = enumeration(
            "Result",
            "Ok",
            vec![Value::Array(vec![Value::Integer(1)].into())],
        );

        assert_eq!(left, right);
    }

    #[test]
    fn partial_eq_preserves_dict_and_record_field_order() {
        let dict = Value::dict(vec![
            (Value::Str("name".into()), Value::Integer(1)),
            (Value::Str("age".into()), Value::Integer(2)),
        ]);
        let reordered_dict = Value::dict(vec![
            (Value::Str("age".into()), Value::Integer(2)),
            (Value::Str("name".into()), Value::Integer(1)),
        ]);
        let original_record = record(
            "Demo.Point",
            &["x", "y"],
            vec![Value::Integer(1), Value::Integer(2)],
        );
        let reordered_record = record(
            "Demo.Point",
            &["y", "x"],
            vec![Value::Integer(2), Value::Integer(1)],
        );

        assert_ne!(dict, reordered_dict);
        assert_ne!(original_record, reordered_record);
    }

    #[test]
    fn aggregate_values_preserve_layout_equality_and_display() {
        let record = Value::Record(SharedRecord::new(
            std::sync::Arc::new(RuntimeRecordLayout {
                record: crate::RecordTypeId::new(0),
                type_name: "Demo.Point".to_string(),
                fields: vec!["x".to_string(), "y".to_string()],
            }),
            vec![Value::Integer(1), Value::Integer(2)],
        ));
        assert_eq!(
            record,
            self::record(
                "Demo.Point",
                &["x", "y"],
                vec![Value::Integer(1), Value::Integer(2)],
            )
        );
        assert_eq!(record.to_string(), "Demo.Point{x: 1, y: 2}");

        let enumeration = Value::Enum(SharedEnum::new(
            std::sync::Arc::new(RuntimeEnumLayout {
                enumeration: crate::EnumTypeId::new(0),
                variant_id: crate::EnumVariantId::new(0),
                type_name: "Demo.Choice".to_string(),
                variant: "Number".to_string(),
                fields: vec!["value".to_string()],
            }),
            vec![Value::Integer(3)],
        ));
        assert_eq!(
            enumeration,
            self::enumeration("Demo.Choice", "Number", vec![Value::Integer(3)])
        );
        assert_eq!(enumeration.to_string(), "Demo.Choice.Number(3)");
    }

    #[test]
    fn partial_eq_compares_function_captures_and_task_binding() {
        let left = Value::function(
            crate::FunctionId::new(1),
            "demo.run".to_string(),
            vec![Value::Integer(1)],
        );
        let right = Value::function(
            crate::FunctionId::new(1),
            "demo.run".to_string(),
            vec![Value::Integer(1)],
        );
        let task_bound = Value::task_owned_function(
            crate::FunctionId::new(1),
            "demo.run".to_string(),
            vec![Value::Integer(1)],
            1,
        );
        let other_owner = Value::task_owned_function(
            crate::FunctionId::new(1),
            "demo.run".to_string(),
            vec![Value::Integer(1)],
            2,
        );

        assert_eq!(left, right);
        assert_ne!(right, task_bound);
        assert_ne!(task_bound, other_owner);
    }

    #[test]
    fn partial_eq_compares_cells_by_shared_allocation() {
        let cell = std::sync::Arc::new(std::sync::Mutex::new(Value::Integer(1)));
        let same_cell = Value::Cell(cell.clone());
        let different_cell = Value::Cell(std::sync::Arc::new(std::sync::Mutex::new(
            Value::Integer(1),
        )));

        assert_eq!(Value::Cell(cell), same_cell);
        assert_ne!(same_cell, different_cell);
    }

    #[test]
    fn runtime_value_stays_compact() {
        assert!(
            std::mem::size_of::<Value>() <= 16,
            "Value grew to {} bytes",
            std::mem::size_of::<Value>()
        );
    }
}
