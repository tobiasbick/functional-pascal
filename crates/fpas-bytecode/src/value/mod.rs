mod equal;

use equal::values_equal;

/// Runtime value in the VM.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Str(String),
    /// Enum variant with optional associated data.
    ///
    /// **Documentation:** `docs/future/advanced-types.md`
    Enum {
        type_name: String,
        variant: String,
        fields: Vec<Value>,
    },
    /// Ordered collection.
    Array(Vec<Value>),
    /// Key-value collection (ordered by insertion).
    ///
    /// **Documentation:** `docs/future/advanced-types.md`
    Dict(Vec<(Value, Value)>),
    /// Record with named fields (field order matches definition).
    Record {
        type_name: String,
        fields: Vec<(String, Value)>,
    },
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
    /// For closures, `captures` holds the values captured from enclosing scopes.
    ///
    /// **Documentation:** `docs/pascal/language/functions/first-class.md`
    Function {
        name: String,
        captures: Vec<Value>,
    },
    /// Task handle (runtime id).
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Task(u64),
}

impl Value {
    /// Return the runtime type category name for diagnostics.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Real(_) => "real",
            Value::Boolean(_) => "boolean",
            Value::Str(_) => "string",
            Value::Enum { .. } => "enum",
            Value::Array(_) => "array",
            Value::Dict(_) => "dict",
            Value::Record { .. } => "record",
            Value::Unit => "unit",
            Value::ResultOk(_) => "Result.Ok",
            Value::ResultError(_) => "Result.Error",
            Value::OptionSome(_) => "Option.Some",
            Value::OptionNone => "Option.None",
            Value::Function { .. } => "function",
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
            Value::Enum {
                type_name,
                variant,
                fields,
            } => {
                write!(f, "{type_name}.{variant}")?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in fields.iter().enumerate() {
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
            Value::Record { type_name, fields } => {
                if type_name == "Std.Tui.ViewId"
                    && let Some(Value::Integer(raw)) = fields
                        .iter()
                        .find(|(name, _)| name == "__id")
                        .map(|(_, value)| value)
                {
                    return write!(f, "{raw}");
                }
                write!(f, "{type_name}{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
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
            Value::Function { name, .. } => write!(f, "<function {name}>"),
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
        assert_eq!(Value::Dict(Vec::new()).type_name(), "dict");
        assert_eq!(Value::Task(9).type_name(), "task");
    }

    #[test]
    fn display_formats_dict_with_braces() {
        let value = Value::Dict(vec![
            (Value::Str("k".into()), Value::Integer(1)),
            (Value::Str("x".into()), Value::Boolean(true)),
        ]);
        assert_eq!(value.to_string(), "{k: 1, x: true}");
    }

    #[test]
    fn display_formats_view_id_record_as_raw_integer() {
        let value = Value::Record {
            type_name: "Std.Tui.ViewId".into(),
            fields: vec![("__id".into(), Value::Integer(42))],
        };
        assert_eq!(value.to_string(), "42");
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
        let left = Value::Array(vec![Value::Array(vec![Value::Integer(1)])]);
        let right = Value::Array(vec![Value::Array(vec![Value::Integer(1)])]);
        assert_eq!(left, right);
    }
}
