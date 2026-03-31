/// Runtime value in the VM.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Char(char),
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
    /// **Documentation:** `docs/pascal/04-functions.md`
    Function {
        name: String,
        captures: Vec<Value>,
    },
    /// Channel handle (runtime id).
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Channel(u64),
    /// Task handle (runtime id).
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Task(u64),
}

impl Value {
    /// Return the runtime type category name for diagnostics.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Real(_) => "real",
            Value::Boolean(_) => "boolean",
            Value::Char(_) => "char",
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
            Value::Channel(_) => "channel",
            Value::Task(_) => "task",
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        if let Value::Real(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::Real(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Char(c) => write!(f, "{c}"),
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
                write!(f, "[")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "]")
            }
            Value::Record { type_name, fields } => {
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
            Value::Channel(id) => write!(f, "<channel {id}>"),
            Value::Task(id) => write!(f, "<task {id}>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Real(a), Self::Real(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (
                Self::Enum {
                    type_name: a_type,
                    variant: a_variant,
                    fields: a_fields,
                },
                Self::Enum {
                    type_name: b_type,
                    variant: b_variant,
                    fields: b_fields,
                },
            ) => a_type == b_type && a_variant == b_variant && a_fields == b_fields,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Dict(a), Self::Dict(b)) => a == b,
            (
                Self::Record {
                    type_name: a_type,
                    fields: a_fields,
                },
                Self::Record {
                    type_name: b_type,
                    fields: b_fields,
                },
            ) => a_type == b_type && a_fields == b_fields,
            (Self::Unit, Self::Unit) => true,
            (Self::ResultOk(a), Self::ResultOk(b)) => a == b,
            (Self::ResultError(a), Self::ResultError(b)) => a == b,
            (Self::OptionSome(a), Self::OptionSome(b)) => a == b,
            (Self::OptionNone, Self::OptionNone) => true,
            (
                Self::Function {
                    name: a_name,
                    captures: a_captures,
                },
                Self::Function {
                    name: b_name,
                    captures: b_captures,
                },
            ) => a_name == b_name && a_captures == b_captures,
            (Self::Channel(a), Self::Channel(b)) => a == b,
            (Self::Task(a), Self::Task(b)) => a == b,
            _ => false,
        }
    }
}
