//! `Std.Str.*` intrinsic implementations (`match` arms).
//!
//! **Documentation:** `docs/pascal/std/text/str/README.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` std call lowering, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_internal_error, std_runtime_error};
use crate::intrinsic_args::{
    IntrinsicCall, expect_str, pad_fill_char, pop_array, pop_int, pop_single_char, pop_string,
    pop_value, value_as_string_for_join,
};
use crate::limits::checked_collection_len;
use crate::numeric_text::is_pascal_numeric;
use fpas_bytecode::{Intrinsic, SourceLocation, StrIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
};

mod format;
mod substring;

/// Runs a `Std.Str` intrinsic if `intrinsic` matches; leaves stack unchanged and returns `Ok(None)` otherwise.
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
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.to_uppercase().into()));
        }
        Intrinsic::Str(StrIntrinsic::ToLower) => {
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.to_lowercase().into()));
        }
        Intrinsic::Str(StrIntrinsic::Trim) => {
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::Contains) => {
            let sub = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(s.contains(sub.as_ref())));
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
            let idx = s
                .find(sub.as_ref())
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            call.push(Value::Integer(idx));
        }
        Intrinsic::Str(StrIntrinsic::Replace) => {
            let new_s = pop_string(pop_value(call, location)?, location)?;
            let old = pop_string(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.replace(&old, &new_s).into()));
        }
        Intrinsic::Str(StrIntrinsic::Split) => {
            let delim = pop_string(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
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
                .map(|p| Value::Str(p.to_string().into()))
                .collect();
            call.push(Value::Array(parts.into()));
        }
        Intrinsic::Str(StrIntrinsic::Join) => {
            let delim = pop_string(pop_value(call, location)?, location)?;
            let arr = pop_array(pop_value(call, location)?, location)?;
            let mut out = String::new();
            for (i, v) in arr.iter().enumerate() {
                let part = value_as_string_for_join(v, location)?;
                if i > 0 {
                    out.push_str(&delim);
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
            let s = pop_string(pop_value(call, location)?, location)?;
            let out = if n <= 0 {
                String::new()
            } else {
                let len = checked_collection_len(n, location, "Std.Str.RepeatStr")?;
                s.repeat(len)
            };
            call.push(Value::Str(out.into()));
        }
        Intrinsic::Str(StrIntrinsic::PadLeft) => {
            let pad_fill = pop_string(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(&pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadLeft", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                call.push(Value::Str(s.into()));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                call.push(Value::Str(format!("{padding}{s}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadRight) => {
            let pad_fill = pop_string(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(&pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadRight", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                call.push(Value::Str(s.into()));
            } else {
                let padding: String = std::iter::repeat_n(pad_char, width - char_count).collect();
                call.push(Value::Str(format!("{s}{padding}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::PadCenter) => {
            let pad_fill = pop_string(pop_value(call, location)?, location)?;
            let pad_char = pad_fill_char(&pad_fill, location)?;
            let width = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
            let width = checked_pad_width(width, "PadCenter", location)?;
            let char_count = s.chars().count();
            if char_count >= width {
                call.push(Value::Str(s.into()));
            } else {
                let total_pad = width - char_count;
                let left_pad = total_pad / 2;
                let right_pad = total_pad - left_pad;
                let lp: String = std::iter::repeat_n(pad_char, left_pad).collect();
                let rp: String = std::iter::repeat_n(pad_char, right_pad).collect();
                call.push(Value::Str(format!("{lp}{s}{rp}").into()));
            }
        }
        Intrinsic::Str(StrIntrinsic::FromChar) => {
            let n = pop_int(pop_value(call, location)?, location)?;
            let c = pop_single_char(pop_value(call, location)?, location)?;
            let len = if n <= 0 {
                0
            } else {
                checked_collection_len(n, location, "Std.Str.FromChar")?
            };
            let s: String = std::iter::repeat_n(c, len).collect();
            call.push(Value::Str(s.into()));
        }
        Intrinsic::Str(StrIntrinsic::CharAt) => {
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let character = usize::try_from(idx)
                .ok()
                .and_then(|index| s.chars().nth(index));
            let Some(character) = character else {
                let length = s.chars().count();
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("CharAt index {idx} out of range (length {length})"),
                    "Ensure the index is within 0..Length(S)-1.",
                    location,
                ));
            };
            call.push(Value::Str(character.to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::SetCharAt) => {
            let c = pop_single_char(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
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
            call.push(Value::Str(chars.into_iter().collect::<String>().into()));
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
            let sub = pop_string(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
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
            call.push(Value::Str(result.into()));
        }
        Intrinsic::Str(StrIntrinsic::Delete) => {
            let len = pop_int(pop_value(call, location)?, location)?;
            let idx = pop_int(pop_value(call, location)?, location)?;
            let s = pop_string(pop_value(call, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            if idx < 0 || len < 0 || idx > n || len > n - idx {
                return Err(std_runtime_error(
                    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
                    format!("Delete out of range (length={n}, index={idx}, count={len})"),
                    "Ensure index and count select a valid range.",
                    location,
                ));
            }
            let end = idx + len;
            let mut result: String = chars[..idx as usize].iter().collect();
            let tail: String = chars[end as usize..].iter().collect();
            result.push_str(&tail);
            call.push(Value::Str(result.into()));
        }
        Intrinsic::Str(StrIntrinsic::Reverse) => {
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.chars().rev().collect::<String>().into()));
        }
        Intrinsic::Str(StrIntrinsic::TrimLeft) => {
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim_start().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::TrimRight) => {
            let s = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(s.trim_end().to_string().into()));
        }
        Intrinsic::Str(StrIntrinsic::LastIndexOf) => {
            let sub = expect_str(pop_value(call, location)?, location)?;
            let s = expect_str(pop_value(call, location)?, location)?;
            let idx = s
                .rfind(sub.as_ref())
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            call.push(Value::Integer(idx));
        }
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
            let template = pop_string(pop_value(call, location)?, location)?;
            let result = format::apply_format(&template, &args, location)?;
            call.push(Value::Str(result.into()));
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
        checked_collection_len(width, location, &format!("Std.Str.{intrinsic_name}"))
    }
}

#[cfg(test)]
mod tests;
