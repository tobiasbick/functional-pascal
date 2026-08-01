//! Stable token and modifier ordering shared by capability and result encoding.

use fpas_language_service::SemanticTokenKind;
use tower_lsp_server::ls_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};

const FIELD: SemanticTokenType = SemanticTokenType::new("field");
const PROCEDURE: SemanticTokenType = SemanticTokenType::new("procedure");
const CONSTANT: SemanticTokenType = SemanticTokenType::new("constant");
const PUBLIC: SemanticTokenModifier = SemanticTokenModifier::new("public");

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
            SemanticTokenType::PROPERTY,
            SemanticTokenType::EVENT,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::FUNCTION,
            PROCEDURE,
            SemanticTokenType::METHOD,
            CONSTANT,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::READONLY,
            PUBLIC,
        ],
    }
}

pub(super) const fn token_type(kind: SemanticTokenKind) -> u32 {
    match kind {
        SemanticTokenKind::Namespace => 0,
        SemanticTokenKind::Type => 1,
        SemanticTokenKind::Enum => 2,
        SemanticTokenKind::TypeParameter => 3,
        SemanticTokenKind::Parameter => 4,
        SemanticTokenKind::Variable => 5,
        SemanticTokenKind::Field => 6,
        SemanticTokenKind::Property => 7,
        SemanticTokenKind::Event => 8,
        SemanticTokenKind::EnumMember => 9,
        SemanticTokenKind::Function => 10,
        SemanticTokenKind::Procedure => 11,
        SemanticTokenKind::Method => 12,
        SemanticTokenKind::Constant => 13,
    }
}
