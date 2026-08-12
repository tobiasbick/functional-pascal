//! Detached Unicode-scalar string character replacement.

use fpas_bytecode::{SharedStr, Value};

use super::model::StringTransformation;
use crate::vm::debug::{DebugErrorKind, DebugSessionError};

/// Replaces one zero-based Unicode scalar with exactly one replacement scalar.
pub(in crate::vm::debug) fn replace_character(
    string: Value,
    index: i64,
    replacement: Value,
) -> Result<StringTransformation, DebugSessionError> {
    let Value::Str(text) = string else {
        return Err(not_string());
    };
    let Value::Str(replacement) = replacement else {
        return Err(character_required());
    };
    let mut replacement_chars = replacement.chars();
    let Some(new_character) = replacement_chars.next() else {
        return Err(character_required());
    };
    if replacement_chars.next().is_some() {
        return Err(character_required());
    }
    let index = usize::try_from(index).map_err(|_| out_of_bounds(index, text.char_len()))?;
    let Some(old_character) = text.chars().nth(index) else {
        return Err(out_of_bounds(index as i64, text.char_len()));
    };
    if old_character == new_character {
        return Err(DebugSessionError {
            kind: DebugErrorKind::StringCharacterUnchanged,
            message: format!("debug string character at index {index} is unchanged"),
            hint: "Use a different single-character string or leave the value unchanged."
                .to_string(),
        });
    }
    let changed = text
        .chars()
        .enumerate()
        .map(|(position, character)| {
            if position == index {
                new_character
            } else {
                character
            }
        })
        .collect::<SharedStr>();
    Ok(StringTransformation {
        string: Value::Str(changed),
        index,
        old_character: Value::Str(old_character.to_string().into()),
        new_character: Value::Str(new_character.to_string().into()),
    })
}

fn not_string() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: "debug string mutation target is not a string".to_string(),
        hint: "Select a mutable target whose complete value is a string.".to_string(),
    }
}

fn character_required() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::StringCharacterRequired,
        message: "debug string replacement must contain exactly one Unicode scalar".to_string(),
        hint: "Use a string expression containing exactly one character.".to_string(),
    }
}

fn out_of_bounds(index: i64, length: usize) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::SequenceIndexOutOfBounds,
        message: format!(
            "debug string index {index} is outside the current character length {length}"
        ),
        hint: "Use a zero-based Unicode scalar index below the current string length.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_unicode_scalars_without_byte_indexing() {
        let original = Value::Str(SharedStr::from("A😀B"));
        let changed = replace_character(original.clone(), 1, Value::Str(SharedStr::from("é")))
            .expect("replace");
        assert_eq!(original, Value::Str(SharedStr::from("A😀B")));
        assert_eq!(changed.string, Value::Str(SharedStr::from("AéB")));
        assert_eq!(changed.old_character, Value::Str(SharedStr::from("😀")));
        let Value::Str(changed_text) = changed.string else {
            panic!("expected string")
        };
        assert_eq!(changed_text.char_len(), 3);
    }

    #[test]
    fn replaces_first_middle_and_last_ascii_scalars() {
        for (index, replacement, expected) in [(0, "x", "xbc"), (1, "x", "axc"), (2, "x", "abx")] {
            let changed = replace_character(
                Value::Str("abc".into()),
                index,
                Value::Str(replacement.into()),
            )
            .expect("replace ASCII scalar");
            assert_eq!(changed.string, Value::Str(expected.into()));
        }
    }

    #[test]
    fn replaces_bmp_and_non_bmp_scalars() {
        let bmp = replace_character(Value::Str("AéB".into()), 1, Value::Str("ø".into()))
            .expect("replace BMP scalar");
        assert_eq!(bmp.string, Value::Str("AøB".into()));
        let non_bmp = replace_character(Value::Str("A😀B".into()), 1, Value::Str("🚒".into()))
            .expect("replace non-BMP scalar");
        assert_eq!(non_bmp.string, Value::Str("A🚒B".into()));
    }

    #[test]
    fn rejects_wrong_lengths_bounds_and_no_op() {
        for replacement in ["", "ab"] {
            assert_eq!(
                replace_character(Value::Str("abc".into()), 0, Value::Str(replacement.into()))
                    .expect_err("length")
                    .kind,
                DebugErrorKind::StringCharacterRequired
            );
        }
        assert_eq!(
            replace_character(Value::Str("abc".into()), 3, Value::Str("x".into()))
                .expect_err("bound")
                .kind,
            DebugErrorKind::SequenceIndexOutOfBounds
        );
        assert_eq!(
            replace_character(Value::Str("abc".into()), 0, Value::Str("a".into()))
                .expect_err("no op")
                .kind,
            DebugErrorKind::StringCharacterUnchanged
        );
    }
}
