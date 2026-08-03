//! UTF-16 relative semantic-token encoding.

use fpas_language_service::{DocumentSnapshot, SemanticToken as FpasToken, SemanticTokenModifiers};
use tower_lsp_server::ls_types::{SemanticToken, SemanticTokens};

use super::legend::token_type;
use crate::convert::{PositionConversionError, byte_offset_to_position};

pub(crate) fn semantic_tokens(
    snapshot: &DocumentSnapshot,
    values: &[FpasToken],
) -> Result<SemanticTokens, PositionConversionError> {
    let mut previous_line = 0;
    let mut previous_start = 0;
    let mut data = Vec::with_capacity(values.len());
    for value in values {
        let start = byte_offset_to_position(snapshot, value.span.offset())?;
        let end = byte_offset_to_position(snapshot, value.span.end())?;
        let delta_line = start.line.saturating_sub(previous_line);
        let delta_start = if delta_line == 0 {
            start.character.saturating_sub(previous_start)
        } else {
            start.character
        };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: end.character.saturating_sub(start.character),
            token_type: token_type(value.kind),
            token_modifiers_bitset: modifier_bits(value.modifiers),
        });
        previous_line = start.line;
        previous_start = start.character;
    }
    Ok(SemanticTokens {
        result_id: None,
        data,
    })
}

fn modifier_bits(modifiers: SemanticTokenModifiers) -> u32 {
    u32::from(modifiers.declaration)
        | (u32::from(modifiers.readonly) << 1)
        | (u32::from(modifiers.public) << 2)
}
