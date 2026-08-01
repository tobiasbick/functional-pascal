//! Protocol-independent semantic highlighting and deterministic source actions.

mod code_actions;
mod tokens;

use fpas_diagnostics::{DiagnosticCode, SourceSpan};

/// Semantic source category attached to one identifier token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticTokenKind {
    /// Program or unit namespace component.
    Namespace,
    /// Named non-enum type.
    Type,
    /// Named enum type.
    Enum,
    /// Generic routine type parameter.
    TypeParameter,
    /// Formal routine parameter.
    Parameter,
    /// Immutable or mutable source variable.
    Variable,
    /// Record field.
    Field,
    /// Computed record property.
    Property,
    /// Record event.
    Event,
    /// Enum member or associated-data constructor.
    EnumMember,
    /// Function declaration or reference.
    Function,
    /// Procedure declaration or reference.
    Procedure,
    /// Record method declaration or reference.
    Method,
    /// Compile-time constant.
    Constant,
}

/// Proven semantic modifiers for one source identifier.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SemanticTokenModifiers {
    /// The token declares its resolved symbol.
    pub declaration: bool,
    /// The resolved value cannot be assigned.
    pub readonly: bool,
    /// The declaration is public outside its source unit.
    pub public: bool,
}

/// One non-overlapping identifier classification in source order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticToken {
    /// Exact UTF-8 source span of the identifier token.
    pub span: SourceSpan,
    /// Proven semantic category.
    pub kind: SemanticTokenKind,
    /// Proven declaration properties.
    pub modifiers: SemanticTokenModifiers,
}

/// Stable identity of one compiler diagnostic in a specific source snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticIdentity {
    /// Stable `Fxxxx` code.
    pub code: DiagnosticCode,
    /// Compiler message without protocol-specific help rendering.
    pub message: String,
    /// Exact source range that triggered the diagnostic.
    pub span: SourceSpan,
}

/// One deterministic source edit offered by a semantic code action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEdit {
    /// Replaced source range, or a zero-width insertion point.
    pub span: SourceSpan,
    /// Canonical replacement text.
    pub new_text: String,
}

/// One quick fix tied to the exact compiler diagnostic it corrects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCodeAction {
    /// User-facing action title.
    pub title: String,
    /// Current compiler diagnostic that authorizes the edit.
    pub diagnostic: DiagnosticIdentity,
    /// Deterministic edits in this document.
    pub edits: Vec<SemanticEdit>,
}
