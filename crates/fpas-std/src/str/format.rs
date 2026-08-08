//! `Std.Str.Format` template expansion.
//!
//! **Documentation:** `docs/pascal/std/text/str/README.md`

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_FORMAT_MISMATCH;

/// Applies printf-style format specifiers (`%d`, `%f`, `%s`, `%%`) to `args`.
///
/// **Documentation:** `docs/pascal/std/text/str/README.md`
pub(super) fn apply_format(
    template: &str,
    args: &[Value],
    location: SourceLocation,
) -> Result<String, StdError> {
    let mut out = String::with_capacity(template.len());
    let mut arg_iter = args.iter();
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            return Err(std_runtime_error(
                RUNTIME_FORMAT_MISMATCH,
                "Format: trailing `%` at end of template string",
                "Escape a literal percent sign as `%%`.",
                location,
            ));
        }
        match chars[i] {
            '%' => {
                out.push('%');
            }
            'd' => {
                let arg = arg_iter.next().ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_FORMAT_MISMATCH,
                        "Format: not enough arguments for `%d` specifier",
                        "Add the missing integer argument or remove the specifier.",
                        location,
                    )
                })?;
                match arg {
                    Value::Integer(n) => out.push_str(&n.to_string()),
                    _ => {
                        return Err(std_runtime_error(
                            RUNTIME_FORMAT_MISMATCH,
                            format!(
                                "Format: `%d` expects an integer, got {}",
                                value_type_name(arg)
                            ),
                            "Pass an integer value for the `%d` specifier.",
                            location,
                        ));
                    }
                }
            }
            'f' => {
                let arg = arg_iter.next().ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_FORMAT_MISMATCH,
                        "Format: not enough arguments for `%f` specifier",
                        "Add the missing real argument or remove the specifier.",
                        location,
                    )
                })?;
                match arg {
                    Value::Real(r) => out.push_str(&format_real(*r)),
                    Value::Integer(n) => out.push_str(&format_real(*n as f64)),
                    _ => {
                        return Err(std_runtime_error(
                            RUNTIME_FORMAT_MISMATCH,
                            format!(
                                "Format: `%f` expects a real or integer, got {}",
                                value_type_name(arg)
                            ),
                            "Pass a real value for the `%f` specifier.",
                            location,
                        ));
                    }
                }
            }
            's' => {
                let arg = arg_iter.next().ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_FORMAT_MISMATCH,
                        "Format: not enough arguments for `%s` specifier",
                        "Add the missing string argument or remove the specifier.",
                        location,
                    )
                })?;
                match arg {
                    Value::Str(s) => out.push_str(s),
                    _ => {
                        return Err(std_runtime_error(
                            RUNTIME_FORMAT_MISMATCH,
                            format!(
                                "Format: `%s` expects a string, got {}",
                                value_type_name(arg)
                            ),
                            "Pass a string value for the `%s` specifier.",
                            location,
                        ));
                    }
                }
            }
            other => {
                return Err(std_runtime_error(
                    RUNTIME_FORMAT_MISMATCH,
                    format!("Format: unknown specifier `%{other}`"),
                    "Supported specifiers: `%d` (integer), `%f` (real), `%s` (string), `%%` (literal %).",
                    location,
                ));
            }
        }
        i += 1;
    }
    if arg_iter.next().is_some() {
        return Err(std_runtime_error(
            RUNTIME_FORMAT_MISMATCH,
            "Format: more arguments than format specifiers",
            "Remove the extra argument or add a matching specifier to the template.",
            location,
        ));
    }
    Ok(out)
}

fn format_real(r: f64) -> String {
    if r.fract() == 0.0 && r.is_finite() {
        format!("{r:.1}")
    } else {
        let s = format!("{r}");
        if s.contains('.') { s } else { format!("{s}.0") }
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Integer(_) => "integer",
        Value::Real(_) => "real",
        Value::Boolean(_) => "boolean",
        Value::Str(_) => "string",
        Value::Array(_) => "array",
        Value::Dict(_) => "dict",
        Value::Record(_) => "record",
        Value::Enum(_) => "enum",
        Value::Unit => "unit",
        Value::ResultOk(_) | Value::ResultError(_) => "result",
        Value::OptionSome(_) | Value::OptionNone => "option",
        Value::Function(_) => "function",
        Value::Cell(_) => "cell",
        Value::Task(_) => "task",
        Value::OpaqueHandle(_) => "opaque handle",
    }
}
