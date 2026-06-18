//! `Std.Str.*` intrinsic implementations (`match` arms).
//!
//! **Documentation:** `docs/pascal/std/text/str.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` std call lowering, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_internal_error, std_runtime_error};
use crate::intrinsic_args::{
    pop_array, pop_char, pop_int, pop_string, pop_value, value_as_string_for_join,
};
use crate::numeric_text::is_pascal_numeric;
use fpas_bytecode::{Intrinsic, SourceLocation, StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_FORMAT_MISMATCH, RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
};

/// Runs a `Std.Str` intrinsic if `intrinsic` matches; leaves stack unchanged and returns `Ok(None)` otherwise.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Str(StrIntrinsic::Length) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(s.chars().count() as i64));
        }
        Intrinsic::Str(StrIntrinsic::ToUpper) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.to_uppercase()));
        }
        Intrinsic::Str(StrIntrinsic::ToLower) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.to_lowercase()));
        }
        Intrinsic::Str(StrIntrinsic::Trim) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim().to_string()));
        }
        Intrinsic::Str(StrIntrinsic::Contains) => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.contains(&sub)));
        }
        Intrinsic::Str(StrIntrinsic::StartsWith) => {
            let pre = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.starts_with(&pre)));
        }
        Intrinsic::Str(StrIntrinsic::EndsWith) => {
            let suf = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.ends_with(&suf)));
        }
        Intrinsic::Str(StrIntrinsic::Substring) => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let start = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            if start < 0 || len < 0 || start > n || start + len > n {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Substring out of range (len={n}, start={start}, len_param={len})"),
                    "Ensure `start` and `len` select a valid substring range.",
                    location,
                ));
            }
            let out: String = chars[start as usize..(start + len) as usize]
                .iter()
                .collect();
            stack.push(Value::Str(out));
        }
        Intrinsic::Str(StrIntrinsic::IndexOf) => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let idx = s
                .find(&sub)
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
        }
        Intrinsic::Str(StrIntrinsic::Replace) => {
            let new_s = pop_string(pop_value(stack, location)?, location)?;
            let old = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.replace(&old, &new_s)));
        }
        Intrinsic::Str(StrIntrinsic::Split) => {
            let delim = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            if delim.is_empty() {
                return Err(std_runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Split delimiter must not be empty",
                    "Pass a non-empty delimiter string to Std.Str.Split.",
                    location,
                ));
            }
            let parts: Vec<Value> = s
                .split(&delim[..])
                .map(|p| Value::Str(p.to_string()))
                .collect();
            stack.push(Value::Array(parts));
        }
        Intrinsic::Str(StrIntrinsic::Join) => {
            let delim = pop_string(pop_value(stack, location)?, location)?;
            let arr = pop_array(pop_value(stack, location)?, location)?;
            let mut out = String::new();
            for (i, v) in arr.iter().enumerate() {
                let part = value_as_string_for_join(v, location)?;
                if i > 0 {
                    out.push_str(&delim);
                }
                out.push_str(&part);
            }
            stack.push(Value::Str(out));
        }
        Intrinsic::Str(StrIntrinsic::IsNumeric) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(is_pascal_numeric(&s)));
        }
        Intrinsic::Str(StrIntrinsic::Repeat) => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let out = if n <= 0 {
                String::new()
            } else {
                s.repeat(n as usize)
            };
            stack.push(Value::Str(out));
        }
        Intrinsic::Str(StrIntrinsic::PadLeft) => {
            let pad_char = pop_char(pop_value(stack, location)?, location)?;
            let width = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let width = checked_pad_width(width, "PadLeft", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                stack.push(Value::Str(s));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                stack.push(Value::Str(format!("{padding}{s}")));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadRight) => {
            let pad_char = pop_char(pop_value(stack, location)?, location)?;
            let width = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let width = checked_pad_width(width, "PadRight", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                stack.push(Value::Str(s));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                stack.push(Value::Str(format!("{s}{padding}")));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadCenter) => {
            let pad_char = pop_char(pop_value(stack, location)?, location)?;
            let width = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let width = checked_pad_width(width, "PadCenter", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                stack.push(Value::Str(s));
            } else {
                let total_pad = width - char_count;
                let left_pad = total_pad / 2;
                let right_pad = total_pad - left_pad;
                let lp: String = std::iter::repeat_n(pad_char, left_pad).collect();
                let rp: String = std::iter::repeat_n(pad_char, right_pad).collect();
                stack.push(Value::Str(format!("{lp}{s}{rp}")));
            }
        }
        Intrinsic::Str(StrIntrinsic::FromChar) => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let c = pop_char(pop_value(stack, location)?, location)?;
            if n < 0 {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!("FromChar count must be >= 0, got {n}"),
                    "Pass a non-negative integer to Std.Str.FromChar.",
                    location,
                ));
            }
            let s: String = std::iter::repeat_n(c, n as usize).collect();
            stack.push(Value::Str(s));
        }
        Intrinsic::Str(StrIntrinsic::CharAt) => {
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx >= chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("CharAt index {idx} out of range (length {})", chars.len()),
                    "Ensure the index is within 0..Length(S)-1.",
                    location,
                ));
            }
            stack.push(Value::Char(chars[idx as usize]));
        }
        Intrinsic::Str(StrIntrinsic::SetCharAt) => {
            let c = pop_char(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let mut chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx >= chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!(
                        "SetCharAt index {idx} out of range (length {})",
                        chars.len()
                    ),
                    "Ensure the index is within 0..Length(S)-1.",
                    location,
                ));
            }
            chars[idx as usize] = c;
            stack.push(Value::Str(chars.into_iter().collect()));
        }
        Intrinsic::Str(StrIntrinsic::Ord) => {
            let c = pop_char(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(c as i64));
        }
        Intrinsic::Str(StrIntrinsic::Chr) => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let c = u32::try_from(n)
                .ok()
                .and_then(char::from_u32)
                .ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_NUMERIC_DOMAIN_ERROR,
                        format!("Chr: {n} is not a valid Unicode code point"),
                        "Pass a valid Unicode code point (0..=0x10FFFF, excluding surrogates).",
                        location,
                    )
                })?;
            stack.push(Value::Char(c));
        }
        Intrinsic::Str(StrIntrinsic::Insert) => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx > chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Insert index {idx} out of range (length {})", chars.len()),
                    "Ensure the index is within 0..Length(S).",
                    location,
                ));
            }
            let byte_offset: usize = chars[..idx as usize].iter().map(|c| c.len_utf8()).sum();
            let mut result = s;
            result.insert_str(byte_offset, &sub);
            stack.push(Value::Str(result));
        }
        Intrinsic::Str(StrIntrinsic::Delete) => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            if idx < 0 || len < 0 || idx > n || idx + len > n {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Delete out of range (length={n}, index={idx}, count={len})"),
                    "Ensure index and count select a valid range.",
                    location,
                ));
            }
            let mut result: String = chars[..idx as usize].iter().collect();
            let tail: String = chars[(idx + len) as usize..].iter().collect();
            result.push_str(&tail);
            stack.push(Value::Str(result));
        }
        Intrinsic::Str(StrIntrinsic::Reverse) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.chars().rev().collect()));
        }
        Intrinsic::Str(StrIntrinsic::TrimLeft) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim_start().to_string()));
        }
        Intrinsic::Str(StrIntrinsic::TrimRight) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim_end().to_string()));
        }
        Intrinsic::Str(StrIntrinsic::LastIndexOf) => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let idx = s
                .rfind(&sub)
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
        }
        Intrinsic::Str(StrIntrinsic::Format) => {
            let arg_count = pop_int(pop_value(stack, location)?, location)?;
            if arg_count < 0 {
                return Err(std_internal_error(
                    "Format: internal error — negative argument count",
                    "Report this as a compiler bug.",
                    location,
                ));
            }
            let mut args: Vec<Value> = Vec::with_capacity(arg_count as usize);
            for _ in 0..arg_count {
                args.push(pop_value(stack, location)?);
            }
            args.reverse();
            let template = pop_string(pop_value(stack, location)?, location)?;
            let result = apply_format(&template, &args, location)?;
            stack.push(Value::Str(result));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn checked_pad_width(
    width: i64,
    intrinsic_name: &str,
    location: SourceLocation,
) -> Result<usize, StdError> {
    if width < 0 {
        Err(std_runtime_error(
            RUNTIME_NUMERIC_DOMAIN_ERROR,
            format!("{intrinsic_name} width must be >= 0, got {width}"),
            format!("Pass a non-negative width to Std.Str.{intrinsic_name}."),
            location,
        ))
    } else {
        Ok(width as usize)
    }
}

/// Applies printf-style format specifiers (`%d`, `%f`, `%s`, `%%`) to `args`.
///
/// **Documentation:** `docs/pascal/std/text/str.md`
fn apply_format(
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
                    Value::Char(c) => out.push(*c),
                    _ => {
                        return Err(std_runtime_error(
                            RUNTIME_FORMAT_MISMATCH,
                            format!(
                                "Format: `%s` expects a string or char, got {}",
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
        // Trim trailing zeros while keeping at least one decimal place.
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
        Value::Char(_) => "char",
        Value::Array(_) => "array",
        Value::Dict(_) => "dict",
        Value::Record { .. } => "record",
        Value::Enum { .. } => "enum",
        Value::Unit => "unit",
        Value::ResultOk(_) | Value::ResultError(_) => "result",
        Value::OptionSome(_) | Value::OptionNone => "option",
        Value::Function { .. } => "function",
        Value::Task(_) => "task",
    }
}
