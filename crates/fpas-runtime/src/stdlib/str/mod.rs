//! `Std.Str.*` intrinsic implementations (`match` arms).
//!
//! **Documentation:** `docs/pascal/language/types/string/README.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas_code::Intrinsic`, `fpas-compiler` std call lowering, and `fpas-sema` `std_registry.rs`.

use crate::stdlib::error::{StdError, std_internal_error, std_runtime_error};
use crate::stdlib::intrinsic_args::{
    IntrinsicCall, expect_array, expect_str, pad_fill_char, pop_int, pop_single_char, pop_value,
    value_as_string_for_join,
};
use crate::stdlib::limits::checked_collection_len;
use crate::{Intrinsic, SourceLocation, StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
};
use fpas_numeric::text::is_numeric as is_pascal_numeric;

mod format;
mod substring;
mod utf8;

/// Runs a `Std.Str` intrinsic if `intrinsic` matches; leaves stack unchanged and returns `Ok(None)` otherwise.
///
/// **Documentation:** `docs/pascal/language/types/string/format-chars.md`,
/// `docs/pascal/language/types/string/edit.md`.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Str(StrIntrinsic::Length) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            // Character count is cached on SharedStr (set at construction / ConcatStr).
            call.push(Value::Integer(s.char_len() as i64));
        }
        Intrinsic::Str(StrIntrinsic::ToUpper) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.to_uppercase().into()));
        }
        Intrinsic::Str(StrIntrinsic::ToLower) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.to_lowercase().into()));
        }
        Intrinsic::Str(StrIntrinsic::Trim) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::StartsWith) => {
            let pre = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(s.starts_with(pre.as_ref())));
        }
        Intrinsic::Str(StrIntrinsic::EndsWith) => {
            let suf = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(s.ends_with(suf.as_ref())));
        }
        Intrinsic::Str(StrIntrinsic::Substring) => {
            let len = pop_int(pop_value(call, location)?, location)?;
            let start = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let out = substring::extract(s, start, len, location)?;
            call.push(Value::Str(out));
        }
        Intrinsic::Str(StrIntrinsic::IndexOf) => {
            let sub = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let idx = s.find(sub.as_ref()).map_or(Value::OptionNone, |b| {
                Value::option_some(Value::Integer(s[..b].chars().count() as i64))
            });
            call.push(idx);
        }
        Intrinsic::Str(StrIntrinsic::Replace) => {
            let new_s = expect_str(pop_value(call, location)?, location)?;
            let old = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.replace(old.as_ref(), new_s.as_ref()).into()));
        }
        Intrinsic::Str(StrIntrinsic::Split) => {
            let delim = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            if delim.is_empty() {
                return Err(std_runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Split delimiter must not be empty",
                    "Pass a non-empty delimiter to Text.Split(Delim).",
                    location,
                ));
            }
            let parts: Vec<Value> = s
                .split(&delim[..])
                .map(|p| Value::Str(p.to_string().into()))
                .collect();
            call.push(Value::Array(parts.into()));
        }
        Intrinsic::Str(StrIntrinsic::Join) => {
            let delim = expect_str(pop_value(call, location)?, location)?;
            let arr = expect_array(pop_value(call, location)?, location)?;
            let mut out = String::new();
            for (i, v) in arr.iter().enumerate() {
                let part = value_as_string_for_join(v, location)?;
                if i > 0 {
                    out.push_str(delim);
                }
                out.push_str(part);
            }
            call.push(Value::Str(out.into()));
        }
        Intrinsic::Str(StrIntrinsic::IsNumeric) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(is_pascal_numeric(s)));
        }
        Intrinsic::Str(StrIntrinsic::Repeat) => {
            let n = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let out = if n <= 0 {
                String::new()
            } else {
                let len = checked_collection_len(n, location, "Std.Str.RepeatStr")?;
                s.repeat(len)
            };
            call.push(Value::Str(out.into()));
        }
        Intrinsic::Str(StrIntrinsic::PadLeft) => {
            let pad_fill = expect_str(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadLeft", location)?;
            let char_count = s.char_len();
            if char_count >= width {
                call.push(Value::Str(s.clone()));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                call.push(Value::Str(format!("{padding}{s}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadRight) => {
            let pad_fill = expect_str(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadRight", location)?;
            let char_count = s.char_len();
            if char_count >= width {
                call.push(Value::Str(s.clone()));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                call.push(Value::Str(format!("{s}{padding}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadCenter) => {
            let pad_fill = expect_str(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadCenter", location)?;
            let char_count = s.char_len();
            if char_count >= width {
                call.push(Value::Str(s.clone()));
            } else {
                let total_pad = width - char_count;
                let left_pad = total_pad / 2;
                let right_pad = total_pad - left_pad;
                let lp: String = std::iter::repeat_n(pad_char, left_pad).collect();
                let rp: String = std::iter::repeat_n(pad_char, right_pad).collect();
                call.push(Value::Str(format!("{lp}{s}{rp}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::SetCharAt) => {
            let c = pop_single_char(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let length = s.char_len();
            if idx < 0 || idx >= length as i64 {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("SetCharAt index {idx} out of range (length {length})"),
                    character_index_hint(length),
                    location,
                ));
            }
            let start = s.byte_offset(idx as usize).unwrap_or(s.len());
            let end = s.byte_offset(idx as usize + 1).unwrap_or(s.len());
            let mut result = String::with_capacity(s.len() - (end - start) + c.len_utf8());
            result.push_str(&s[..start]);
            result.push(c);
            result.push_str(&s[end..]);
            call.push(Value::Str(result.into()));
        }
        Intrinsic::Str(StrIntrinsic::Ord) => {
            let c = pop_single_char(pop_value(call, location)?, location)?;
            call.push(Value::Integer(c as i64));
        }
        Intrinsic::Str(StrIntrinsic::Chr) => {
            let n = pop_int(pop_value(call, location)?, location)?;
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
            call.push(Value::Str(c.to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::Insert) => {
            let sub = expect_str(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let length = s.char_len();
            if idx < 0 || idx > length as i64 {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Insert index {idx} out of range (length {length})"),
                    "Ensure the index is within 0..S.Length().",
                    location,
                ));
            }
            let byte_offset = s.byte_offset(idx as usize).unwrap_or(s.len());
            let mut result = String::with_capacity(s.len() + sub.len());
            result.push_str(&s[..byte_offset]);
            result.push_str(sub);
            result.push_str(&s[byte_offset..]);
            call.push(Value::Str(result.into()));
        }
        Intrinsic::Str(StrIntrinsic::Delete) => {
            let len = pop_int(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let n = s.char_len() as i64;
            if idx < 0 || len < 0 || idx > n || len > n - idx {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Delete out of range (length={n}, index={idx}, count={len})"),
                    "Ensure index and count select a valid range.",
                    location,
                ));
            }
            let start = s.byte_offset(idx as usize).unwrap_or(s.len());
            let end = s.byte_offset((idx + len) as usize).unwrap_or(s.len());
            let mut result = String::with_capacity(s.len() - (end - start));
            result.push_str(&s[..start]);
            result.push_str(&s[end..]);
            call.push(Value::Str(result.into()));
        }
        Intrinsic::Str(StrIntrinsic::Reverse) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.chars().rev().collect::<String>().into()));
        }
        Intrinsic::Str(StrIntrinsic::TrimLeft) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim_start().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::TrimRight) => {
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim_end().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::LastIndexOf) => {
            let sub = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let idx = s.rfind(sub.as_ref()).map_or(Value::OptionNone, |b| {
                Value::option_some(Value::Integer(s[..b].chars().count() as i64))
            });
            call.push(idx);
        }
        Intrinsic::Str(
            operation @ (StrIntrinsic::Utf8Encode
            | StrIntrinsic::Utf8Decode
            | StrIntrinsic::Utf8Append),
        ) => utf8::run(operation, call, location)?,
        Intrinsic::Str(StrIntrinsic::Format) => {
            let arg_count = pop_int(pop_value(call, location)?, location)?;
            if arg_count < 0 {
                return Err(std_internal_error(
                    "Format: internal error — negative argument count",
                    "Report this as a compiler bug.",
                    location,
                ));
            }
            let mut args: Vec<Value> = Vec::with_capacity(arg_count as usize);
            for _ in 0..arg_count {
                args.push(pop_value(call, location)?.clone());
            }
            args.reverse();
            let template = expect_str(pop_value(call, location)?, location)?;
            let result = format::apply_format(template, &args, location)?;
            call.push(Value::Str(result.into()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn character_index_hint(length: usize) -> &'static str {
    if length == 0 {
        "An empty string has no valid character index. Check S.IsEmpty() before calling this operation."
    } else {
        "Ensure the index is within 0..S.Length()-1."
    }
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
            format!("Pass a non-negative width to Text.{intrinsic_name}(Width, PadChar)."),
            location,
        ))
    } else {
        checked_collection_len(width, location, &format!("Std.Str.{intrinsic_name}"))
    }
}

#[cfg(test)]
mod tests;
