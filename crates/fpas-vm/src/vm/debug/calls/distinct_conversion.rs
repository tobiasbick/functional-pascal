//! Identity conversions into and out of distinct types during debugger evaluation.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{DebugType, Value};

use super::detach::error;
use super::execute::CallSandbox;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

const SCALAR_NAMES: [&str; 4] = ["integer", "real", "string", "boolean"];

impl CallSandbox {
    /// Returns the scalar runtime type name required by a conversion call, if `name` is one.
    ///
    /// Distinct types visible in the evaluated source wrap their underlying type;
    /// the built-in scalar type names unwrap. Distinct values share their underlying
    /// runtime representation, so both directions check only the value's scalar type.
    pub(super) fn conversion_type(&self, name: &str) -> Option<&'static str> {
        let image = self.executable.executable();
        let visible = image.distinct_types.iter().find(|distinct| {
            distinct.source == self.source
                && image
                    .strings
                    .get(distinct.name)
                    .is_some_and(|visible| visible.eq_ignore_ascii_case(name))
        });
        if let Some(distinct) = visible {
            return match image.debug_types.get(distinct.underlying.get() as usize)? {
                DebugType::Integer => Some("integer"),
                DebugType::Real => Some("real"),
                DebugType::String => Some("string"),
                DebugType::Boolean => Some("boolean"),
                _ => None,
            };
        }
        SCALAR_NAMES
            .into_iter()
            .find(|scalar| scalar.eq_ignore_ascii_case(name))
    }

    /// Returns the single argument unchanged when its runtime type is `expected`.
    pub(super) fn convert(
        &self,
        name: &str,
        expected: &str,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        let count = arguments.len();
        let Ok([value]) = <[Value; 1]>::try_from(arguments) else {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!("debug conversion `{name}(...)` expects 1 argument, got {count}"),
                format!("Pass exactly one value, for example `{name}(Value)`."),
            ));
        };
        if value.type_name() == expected {
            return Ok(value);
        }
        Err(error(
            DebugErrorKind::EvaluationType,
            format!(
                "debug conversion `{name}(...)` requires a {expected} value, got {}",
                value.type_name()
            ),
            format!("Pass a value whose runtime type is {expected}."),
        ))
    }
}
