mod equal;

use equal::values_equal;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Copy-on-write storage for FPAS array values.
///
/// Cloning an array shares its elements until a mutable operation occurs. This preserves FPAS
/// value semantics while avoiding deep copies for ordinary reads of large arrays.
#[derive(Debug, Clone)]
pub struct SharedArray(Arc<Vec<Value>>);

impl From<Vec<Value>> for SharedArray {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(values))
    }
}

impl FromIterator<Value> for SharedArray {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl From<SharedArray> for Vec<Value> {
    fn from(values: SharedArray) -> Self {
        Arc::unwrap_or_clone(values.0)
    }
}

impl Deref for SharedArray {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl<'a> IntoIterator for &'a SharedArray {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for SharedArray {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}

/// UTF-8 payload plus a cached Unicode scalar count for O(1) [`SharedStr::char_len`].
#[derive(Debug, Clone)]
struct StrBody {
    data: String,
    char_len: usize,
}

impl PartialEq for StrBody {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for StrBody {}

impl PartialOrd for StrBody {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrBody {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl std::hash::Hash for StrBody {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

/// Shared immutable storage for FPAS string values.
///
/// Cloning a string shares its UTF-8 buffer and cached character length, avoiding a deep copy
/// until an owning consumer needs to mutate the string. [`SharedStr::char_len`] is O(1).
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SharedStr(Arc<StrBody>);

impl SharedStr {
    /// Unicode scalar count (`Std.Str.Length`), cached at construction and concat time.
    ///
    /// **Documentation:** `docs/pascal/std/text/str/case-trim.md` (Length); contributor map in
    /// `docs/pascal/std/text/str/README.md`.
    pub fn char_len(&self) -> usize {
        self.0.char_len
    }

    /// Concatenate two shared strings, summing cached character lengths.
    pub fn concat(left: &Self, right: &Self) -> Self {
        let mut data = String::with_capacity(left.len() + right.len());
        data.push_str(left);
        data.push_str(right);
        Self(Arc::new(StrBody {
            data,
            char_len: left.char_len() + right.char_len(),
        }))
    }

    fn from_parts(data: String, char_len: usize) -> Self {
        Self(Arc::new(StrBody { data, char_len }))
    }
}

fn count_chars(value: &str) -> usize {
    if value.is_ascii() {
        value.len()
    } else {
        value.chars().count()
    }
}

impl From<String> for SharedStr {
    fn from(value: String) -> Self {
        let char_len = count_chars(&value);
        Self::from_parts(value, char_len)
    }
}

impl From<&str> for SharedStr {
    fn from(value: &str) -> Self {
        Self::from(value.to_owned())
    }
}

impl From<SharedStr> for String {
    fn from(value: SharedStr) -> Self {
        Arc::unwrap_or_clone(value.0).data
    }
}

impl FromIterator<char> for SharedStr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let chars: Vec<char> = iter.into_iter().collect();
        let char_len = chars.len();
        Self::from_parts(chars.into_iter().collect(), char_len)
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.data.as_str()
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl std::fmt::Display for SharedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

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
    Enum {
        type_name: String,
        variant: String,
        fields: Vec<Value>,
    },
    /// Ordered collection.
    Array(SharedArray),
    /// Key-value collection (ordered by insertion).
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
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
    /// For closures, `captures` holds values or [`Value::Cell`] handles captured from
    /// enclosing scopes. `task_bound` is true when any capture is a mutable cell.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    Function {
        name: String,
        captures: Vec<Value>,
        task_bound: bool,
    },
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
            Value::Cell(cell) => match cell.lock() {
                Ok(guard) => write!(f, "<cell {guard}>"),
                Err(_) => write!(f, "<cell <poisoned>>"),
            },
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
        let left = Value::Enum {
            type_name: "Result".to_string(),
            variant: "Ok".to_string(),
            fields: vec![Value::Array(vec![Value::Integer(1)].into())],
        };
        let right = Value::Enum {
            type_name: "Result".to_string(),
            variant: "Ok".to_string(),
            fields: vec![Value::Array(vec![Value::Integer(1)].into())],
        };

        assert_eq!(left, right);
    }

    #[test]
    fn partial_eq_preserves_dict_and_record_field_order() {
        let dict = Value::Dict(vec![
            (Value::Str("name".into()), Value::Integer(1)),
            (Value::Str("age".into()), Value::Integer(2)),
        ]);
        let reordered_dict = Value::Dict(vec![
            (Value::Str("age".into()), Value::Integer(2)),
            (Value::Str("name".into()), Value::Integer(1)),
        ]);
        let record = Value::Record {
            type_name: "Demo.Point".to_string(),
            fields: vec![
                ("x".to_string(), Value::Integer(1)),
                ("y".to_string(), Value::Integer(2)),
            ],
        };
        let reordered_record = Value::Record {
            type_name: "Demo.Point".to_string(),
            fields: vec![
                ("y".to_string(), Value::Integer(2)),
                ("x".to_string(), Value::Integer(1)),
            ],
        };

        assert_ne!(dict, reordered_dict);
        assert_ne!(record, reordered_record);
    }

    #[test]
    fn partial_eq_compares_function_captures_and_task_binding() {
        let left = Value::Function {
            name: "demo.run".to_string(),
            captures: vec![Value::Integer(1)],
            task_bound: false,
        };
        let right = Value::Function {
            name: "demo.run".to_string(),
            captures: vec![Value::Integer(1)],
            task_bound: false,
        };
        let task_bound = Value::Function {
            name: "demo.run".to_string(),
            captures: vec![Value::Integer(1)],
            task_bound: true,
        };

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
    fn shared_arrays_copy_only_when_mutated() {
        let original = SharedArray::from(vec![Value::Integer(1)]);
        let mut updated = original.clone();
        assert!(Arc::ptr_eq(&original.0, &updated.0));

        updated[0] = Value::Integer(2);

        assert!(!Arc::ptr_eq(&original.0, &updated.0));
        assert_eq!(original[0], Value::Integer(1));
        assert_eq!(updated[0], Value::Integer(2));
    }

    #[test]
    fn shared_strings_clone_without_copying_utf8() {
        let original = SharedStr::from("hello");
        let cloned = original.clone();

        assert!(Arc::ptr_eq(&original.0, &cloned.0));
        assert_eq!(String::from(cloned), "hello");
    }

    #[test]
    fn shared_strings_cache_char_len_for_ascii_and_unicode() {
        assert_eq!(SharedStr::from("hello").char_len(), 5);
        assert_eq!(SharedStr::from("café").char_len(), 4);
    }

    #[test]
    fn shared_strings_concat_sums_cached_char_len() {
        let left = SharedStr::from("café");
        let right = SharedStr::from("!");
        let joined = SharedStr::concat(&left, &right);
        assert_eq!(joined.as_ref(), "café!");
        assert_eq!(joined.char_len(), 5);
    }
}
