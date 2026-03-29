//! `Std.Str.*` intrinsic implementations (`match` arms).
//!
//! **Documentation:** `docs/pascal/std/str.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` std call lowering, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{
    pop_array, pop_char, pop_int, pop_string, pop_value, value_as_string_for_join,
};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_INTRINSIC_STACK_STATE_ERROR,
};

/// Runs a `Std.Str` intrinsic if `intrinsic` matches; leaves stack unchanged and returns `Ok(None)` otherwise.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::StrLength => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(s.chars().count() as i64));
        }
        Intrinsic::StrToUpper => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.to_uppercase()));
        }
        Intrinsic::StrToLower => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.to_lowercase()));
        }
        Intrinsic::StrTrim => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim().to_string()));
        }
        Intrinsic::StrContains => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.contains(&sub)));
        }
        Intrinsic::StrStartsWith => {
            let pre = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.starts_with(&pre)));
        }
        Intrinsic::StrEndsWith => {
            let suf = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(s.ends_with(&suf)));
        }
        Intrinsic::StrSubstring => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let start = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            if start < 0 || len < 0 || start > n || start + len > n {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
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
        Intrinsic::StrIndexOf => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let idx = s
                .find(&sub)
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
        }
        Intrinsic::StrReplace => {
            let new_s = pop_string(pop_value(stack, location)?, location)?;
            let old = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.replace(&old, &new_s)));
        }
        Intrinsic::StrSplit => {
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
        Intrinsic::StrJoin => {
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
        Intrinsic::StrIsNumeric => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let t = s.trim();
            stack.push(Value::Boolean(
                t.parse::<i64>().is_ok() || t.parse::<f64>().is_ok(),
            ));
        }
        Intrinsic::StrRepeat => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            if n < 0 {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("Repeat count must be >= 0, got {n}"),
                    "Pass a non-negative integer to Std.Str.Repeat.",
                    location,
                ));
            }
            stack.push(Value::Str(s.repeat(n as usize)));
        }
        Intrinsic::StrPadLeft => {
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
        Intrinsic::StrPadRight => {
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
        Intrinsic::StrPadCenter => {
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
        Intrinsic::StrFromChar => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let c = pop_char(pop_value(stack, location)?, location)?;
            if n < 0 {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("FromChar count must be >= 0, got {n}"),
                    "Pass a non-negative integer to Std.Str.FromChar.",
                    location,
                ));
            }
            let s: String = std::iter::repeat_n(c, n as usize).collect();
            stack.push(Value::Str(s));
        }
        Intrinsic::StrCharAt => {
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx >= chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("CharAt index {idx} out of range (length {})", chars.len()),
                    "Ensure the index is within 0..Length(S)-1.",
                    location,
                ));
            }
            stack.push(Value::Char(chars[idx as usize]));
        }
        Intrinsic::StrSetCharAt => {
            let c = pop_char(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let mut chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx >= chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
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
        Intrinsic::StrOrd => {
            let c = pop_char(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(c as i64));
        }
        Intrinsic::StrChr => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            let c = u32::try_from(n)
                .ok()
                .and_then(char::from_u32)
                .ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Chr: {n} is not a valid Unicode code point"),
                        "Pass a valid Unicode code point (0..=0x10FFFF, excluding surrogates).",
                        location,
                    )
                })?;
            stack.push(Value::Char(c));
        }
        Intrinsic::StrInsert => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            if idx < 0 || idx > chars.len() as i64 {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
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
        Intrinsic::StrDelete => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let idx = pop_int(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            if idx < 0 || len < 0 || idx > n || idx + len > n {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
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
        Intrinsic::StrReverse => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.chars().rev().collect()));
        }
        Intrinsic::StrTrimLeft => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim_start().to_string()));
        }
        Intrinsic::StrTrimRight => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(s.trim_end().to_string()));
        }
        Intrinsic::StrLastIndexOf => {
            let sub = pop_string(pop_value(stack, location)?, location)?;
            let s = pop_string(pop_value(stack, location)?, location)?;
            let idx = s
                .rfind(&sub)
                .map(|b| s[..b].chars().count() as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
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
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("{intrinsic_name} width must be >= 0, got {width}"),
            format!("Pass a non-negative width to Std.Str.{intrinsic_name}."),
            location,
        ))
    } else {
        Ok(width as usize)
    }
}
