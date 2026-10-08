//! Stable token and modifier ordering shared by capability and result encoding.

use fpas_language_service::SemanticTokenKind;
use tower_lsp_server::ls_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};

const FIELD: SemanticTokenType = SemanticTokenType::new("field");
const PROCEDURE: SemanticTokenType = SemanticTokenType::new("procedure");
const CONSTANT: SemanticTokenType = SemanticTokenType::new("constant");
const PUBLIC: SemanticTokenModifier = SemanticTokenModifier::new("public");

/// Includes the contextual `as` keyword described in `docs/pascal/tools/editor-integration.md`.
pub(crate) fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::TYPE,
            SemanticTokenType::ENUM,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::VARIABLE,
            FIELD,
            SemanticTokenType::EVENT,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::FUNCTION,
            PROCEDURE,
            SemanticTokenType::METHOD,
            CONSTANT,
            SemanticTokenType::KEYWORD,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::READONLY,
            PUBLIC,
        ],
    }
}

/// Maps service classifications to their stable LSP legend index.
pub(super) const fn token_type(kind: SemanticTokenKind) -> u32 {
    match kind {
        SemanticTokenKind::Namespace => 0,
        SemanticTokenKind::Type => 1,
        SemanticTokenKind::Enum => 2,
        SemanticTokenKind::TypeParameter => 3,
        SemanticTokenKind::Parameter => 4,
        SemanticTokenKind::Variable => 5,
        SemanticTokenKind::Field => 6,
        SemanticTokenKind::Event => 7,
        SemanticTokenKind::EnumMember => 8,
        SemanticTokenKind::Function => 9,
        SemanticTokenKind::Procedure => 10,
        SemanticTokenKind::Method => 11,
        SemanticTokenKind::Constant => 12,
        SemanticTokenKind::Keyword => 13,
    }
}
