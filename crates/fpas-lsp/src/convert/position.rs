//! Conversion between LSP UTF-16 positions and compiler UTF-8 byte offsets.

use std::fmt;

use fpas_language_service::DocumentSnapshot;
use tower_lsp_server::ls_types::Position;

/// A recoverable invalid or unrepresentable source position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionConversionError {
    /// The zero-based line does not exist in the snapshot.
    LineOutOfRange {
        /// Rejected line number.
        line: u32,
    },
    /// The UTF-16 character offset is beyond the line content.
    CharacterOutOfRange {
        /// Line containing the rejected character offset.
        line: u32,
        /// Rejected UTF-16 code-unit offset.
        character: u32,
    },
    /// The UTF-16 position points between the two code units of a surrogate pair.
    InsideSurrogatePair {
        /// Line containing the invalid position.
        line: u32,
        /// Rejected UTF-16 code-unit offset.
        character: u32,
    },
    /// The compiler byte offset is outside the source or inside a UTF-8 character.
    ByteOffsetOutOfRange {
        /// Rejected UTF-8 byte offset.
        offset: usize,
    },
}

impl fmt::Display for PositionConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LineOutOfRange { line } => {
                write!(formatter, "LSP line {line} is outside the document.")
            }
            Self::CharacterOutOfRange { line, character } => write!(
                formatter,
                "LSP UTF-16 character {character} is outside line {line}."
            ),
            Self::InsideSurrogatePair { line, character } => write!(
                formatter,
                "LSP UTF-16 character {character} on line {line} splits a surrogate pair."
            ),
            Self::ByteOffsetOutOfRange { offset } => write!(
                formatter,
                "UTF-8 byte offset {offset} is outside the document or not a character boundary."
            ),
        }
    }
}

impl std::error::Error for PositionConversionError {}

/// Converts an LSP UTF-16 position to a compiler UTF-8 byte offset.
pub fn position_to_byte_offset(
    snapshot: &DocumentSnapshot,
    position: Position,
) -> Result<usize, PositionConversionError> {
    let line =
        usize::try_from(position.line).map_err(|_| PositionConversionError::LineOutOfRange {
            line: position.line,
        })?;
    let range = snapshot
        .line_index()
        .line_range(snapshot.source(), line)
        .ok_or(PositionConversionError::LineOutOfRange {
            line: position.line,
        })?;
    let target = usize::try_from(position.character).map_err(|_| {
        PositionConversionError::CharacterOutOfRange {
            line: position.line,
            character: position.character,
        }
    })?;
    let mut utf16_column = 0;

    for (byte_column, character) in snapshot.source()[range.clone()].char_indices() {
        if utf16_column == target {
            return Ok(range.start + byte_column);
        }
        let next_column = utf16_column + character.len_utf16();
        if target < next_column {
            return Err(PositionConversionError::InsideSurrogatePair {
                line: position.line,
                character: position.character,
            });
        }
        utf16_column = next_column;
    }

    if utf16_column == target {
        Ok(range.end)
    } else {
        Err(PositionConversionError::CharacterOutOfRange {
            line: position.line,
            character: position.character,
        })
    }
}

/// Converts a compiler UTF-8 byte offset to an LSP UTF-16 position.
pub fn byte_offset_to_position(
    snapshot: &DocumentSnapshot,
    offset: usize,
) -> Result<Position, PositionConversionError> {
    let text_position = snapshot
        .line_index()
        .position(snapshot.source(), offset)
        .ok_or(PositionConversionError::ByteOffsetOutOfRange { offset })?;
    let range = snapshot
        .line_index()
        .line_range(snapshot.source(), text_position.line)
        .ok_or(PositionConversionError::ByteOffsetOutOfRange { offset })?;
    let content_offset = offset.min(range.end);
    if content_offset < range.start {
        return Err(PositionConversionError::ByteOffsetOutOfRange { offset });
    }
    let utf16_column = snapshot.source()[range.start..content_offset]
        .encode_utf16()
        .count();

    Ok(Position {
        line: u32::try_from(text_position.line)
            .map_err(|_| PositionConversionError::ByteOffsetOutOfRange { offset })?,
        character: u32::try_from(utf16_column)
            .map_err(|_| PositionConversionError::ByteOffsetOutOfRange { offset })?,
    })
}
