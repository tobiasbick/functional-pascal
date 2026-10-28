//! Native error value builders for an admitted runtime image.

pub use fpas_builtins::native_errors::{ErrorFieldType, ErrorVariant, NativeErrorFamily};

/// Construct a declared native variant through the executing image's layout cache.
///
/// # Errors
/// Returns an internal diagnostic when the admitted image lacks the layout.
pub fn value(
    factory: &dyn crate::stdlib::AggregateFactory,
    family: NativeErrorFamily,
    variant: usize,
    fields: Vec<crate::Value>,
    location: crate::SourceLocation,
) -> Result<crate::Value, crate::stdlib::StdError> {
    factory.native_error(family, variant, fields, location)
}

/// Construct the original native host cause, without interpreting its explanation.
///
/// # Errors
/// Returns an internal diagnostic when the admitted image lacks HostError.
pub fn host(
    factory: &dyn crate::stdlib::AggregateFactory,
    operation: &str,
    resources: &[String],
    code: Option<i32>,
    detail: String,
    location: crate::SourceLocation,
) -> Result<crate::Value, crate::stdlib::StdError> {
    use crate::Value;
    let domain = code.map(|_| Value::Str(if cfg!(windows) { "win32" } else { "errno" }.into()));
    value(
        factory,
        NativeErrorFamily::Host,
        0,
        vec![
            Value::Str(operation.into()),
            Value::Array(
                resources
                    .iter()
                    .map(|resource| Value::Str(resource.clone().into()))
                    .collect(),
            ),
            domain.map_or(Value::OptionNone, Value::option_some),
            code.map_or(Value::OptionNone, |code| {
                Value::option_some(Value::Integer(i64::from(code)))
            }),
            Value::Str(detail.into()),
        ],
        location,
    )
}

/// Build a parser failure, retaining only positions supplied by the parser.
///
/// # Errors
/// Returns an internal diagnostic when the image lacks its declared error family.
pub fn syntax(
    factory: &dyn crate::stdlib::AggregateFactory,
    family: NativeErrorFamily,
    line: Option<usize>,
    column: Option<usize>,
    detail: String,
    location: crate::SourceLocation,
) -> Result<crate::Value, crate::stdlib::StdError> {
    use crate::Value;
    let position = |n: Option<usize>| {
        n.and_then(|n| i64::try_from(n).ok())
            .map_or(Value::OptionNone, |n| Value::option_some(Value::Integer(n)))
    };
    value(
        factory,
        family,
        0,
        vec![position(line), position(column), Value::Str(detail.into())],
        location,
    )
}
