//! Constant-pool values that can be stored in compiled artifacts.

use std::fmt;

use crate::Value;

/// Runtime-independent value supported by persistent bytecode constant pools.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PersistentValue {
    /// Signed integer.
    Integer(i64),
    /// IEEE-754 bit representation.
    Real(u64),
    /// Boolean.
    Boolean(bool),
    /// UTF-8 string.
    String(String),
    /// Procedure result value.
    Unit,
    /// Named non-capturing function value.
    Function {
        /// Canonical callable name.
        name: String,
        /// Whether calls are restricted to the creating task.
        task_bound: bool,
    },
}

impl PersistentValue {
    /// Convert a runtime value into its persistent representation.
    pub fn from_value(value: &Value) -> Result<Self, PersistentValueError> {
        match value {
            Value::Integer(value) => Ok(Self::Integer(*value)),
            Value::Real(value) => Ok(Self::Real(value.to_bits())),
            Value::Boolean(value) => Ok(Self::Boolean(*value)),
            Value::Str(value) => Ok(Self::String(value.to_string())),
            Value::Unit => Ok(Self::Unit),
            Value::Function(function) if function.captures.is_empty() => Ok(Self::Function {
                name: function.name.clone(),
                task_bound: function.task_bound,
            }),
            other => Err(PersistentValueError::UnsupportedRuntimeValue(
                other.type_name().to_string(),
            )),
        }
    }

    /// Convert the persistent representation into its runtime bytecode value.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Integer(value) => Value::Integer(*value),
            Self::Real(bits) => Value::Real(f64::from_bits(*bits)),
            Self::Boolean(value) => Value::Boolean(*value),
            Self::String(value) => Value::Str(value.clone().into()),
            Self::Unit => Value::Unit,
            Self::Function { name, task_bound } => {
                Value::function(name.clone(), Vec::new(), *task_bound)
            }
        }
    }
}

/// A runtime-only value was requested for a persistent constant pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentValueError {
    /// Runtime value category that cannot be serialized into bytecode.
    UnsupportedRuntimeValue(String),
}

impl fmt::Display for PersistentValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRuntimeValue(value_type) => {
                write!(
                    formatter,
                    "runtime value of type `{value_type}` cannot be stored in compiled bytecode"
                )
            }
        }
    }
}

impl std::error::Error for PersistentValueError {}
