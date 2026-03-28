//! `Std.Str.*` intrinsic implementations (`match` arms).
//!
//! **Documentation:** `docs/pascal/std/str.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` std call lowering, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_array, pop_int, pop_string, pop_value, value_as_string_for_join};
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
        _ => return Ok(None),
    }
    Ok(Some(()))
}
