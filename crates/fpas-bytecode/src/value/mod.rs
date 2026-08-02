mod aggregate;
mod array;
mod equal;
mod function;
mod string;

pub use aggregate::{EnumValue, RecordValue, SharedDict, SharedEnum, SharedRecord};
pub use array::SharedArray;
pub(crate) use equal::constant_values_equal;
use equal::values_equal;
pub use function::{FunctionValue, SharedFunction};
pub use string::SharedStr;

/// Runtime value in the VM.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
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
    /// Record with named fields (field order matches definition).
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
}

impl Value {
    /// Create an enum value with shared immutable storage.
    pub fn enum_value(type_name: String, variant: String, fields: Vec<Value>) -> Self {
        Self::Enum(SharedEnum::new(type_name, variant, fields))
    }

    /// Create an ordered dictionary value with copy-on-write storage.
    pub fn dict(pairs: Vec<(Value, Value)>) -> Self {
        Self::Dict(pairs.into())
    }

    /// Create a record value with copy-on-write storage.
    pub fn record(type_name: String, fields: Vec<(String, Value)>) -> Self {
        Self::Record(SharedRecord::new(type_name, fields))
    }

    /// Create a first-class function value with shared immutable storage.
    pub fn function(name: String, captures: Vec<Value>, task_bound: bool) -> Self {
        Self::Function(SharedFunction::new(name, captures, task_bound))
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
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::Real(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Enum(value) => {
                write!(f, "{}.{}", value.type_name, value.variant)?;
                if !value.fields.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in value.fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{v}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            Value::Array(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{e}")?;
                }
                write!(f, "]")
            }
            Value::Dict(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Record(record) => {
                write!(f, "{}{{", record.type_name)?;
                for (i, (name, val)) in record.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {val}")?;
                }
                write!(f, "}}")
            }
            Value::Unit => write!(f, "()"),
            Value::ResultOk(v) => write!(f, "Ok({v})"),
            Value::ResultError(v) => write!(f, "Error({v})"),
            Value::OptionSome(v) => write!(f, "Some({v})"),
            Value::OptionNone => write!(f, "None"),
            Value::Function(function) => write!(f, "<function {}>", function.name),
            Value::Cell(_) => write!(f, "<cell>"),
            Value::Task(id) => write!(f, "<task {id}>"),
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

    #[test]
    fn type_name_reports_runtime_categories() {
        assert_eq!(Value::Integer(1).type_name(), "integer");
        assert_eq!(Value::dict(Vec::new()).type_name(), "dict");
        assert_eq!(Value::Task(9).type_name(), "task");
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
        let left = Value::enum_value(
            "Result".to_string(),
            "Ok".to_string(),
            vec![Value::Array(vec![Value::Integer(1)].into())],
        );
        let right = Value::enum_value(
            "Result".to_string(),
            "Ok".to_string(),
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
        let record = Value::record(
            "Demo.Point".to_string(),
            vec![
                ("x".to_string(), Value::Integer(1)),
                ("y".to_string(), Value::Integer(2)),
            ],
        );
        let reordered_record = Value::record(
            "Demo.Point".to_string(),
            vec![
                ("y".to_string(), Value::Integer(2)),
                ("x".to_string(), Value::Integer(1)),
            ],
        );

        assert_ne!(dict, reordered_dict);
        assert_ne!(record, reordered_record);
    }

    #[test]
    fn partial_eq_compares_function_captures_and_task_binding() {
        let left = Value::function("demo.run".to_string(), vec![Value::Integer(1)], false);
        let right = Value::function("demo.run".to_string(), vec![Value::Integer(1)], false);
        let task_bound = Value::function("demo.run".to_string(), vec![Value::Integer(1)], true);

        assert_eq!(left, right);
        assert_ne!(right, task_bound);
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
