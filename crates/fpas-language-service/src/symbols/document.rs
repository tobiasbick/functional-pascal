//! Public hierarchical declaration-symbol model.

use fpas_diagnostics::SourceSpan;

use crate::DocumentSnapshot;

/// Editor-facing declaration category independent from LSP symbol kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// Program compilation-unit declaration.
    Program,
    /// Unit compilation-unit declaration.
    Unit,
    /// Compile-time constant.
    Constant,
    /// Immutable variable.
    Variable,
    /// Mutable variable.
    MutableVariable,
    /// Named type or alias.
    Type,
    /// Named enum type.
    Enum,
    /// Function declaration.
    Function,
    /// Procedure declaration.
    Procedure,
    /// Record method declaration.
    Method,
    /// Generic routine type parameter.
    TypeParameter,
    /// Formal routine or closure parameter.
    Parameter,
    /// Record field.
    Field,
    /// Record computed property.
    Property,
    /// Record event.
    Event,
    /// Enum member.
    EnumMember,
    /// Loop-local binding.
    LoopVariable,
}

/// Cross-document visibility of a declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolVisibility {
    /// The declaration is visible to importing units.
    Public,
    /// The declaration is visible only inside its defining source.
    Private,
}

/// One editor-facing callable signature attached to a routine or callable value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableSignature {
    /// Complete Functional Pascal signature shown by the editor.
    pub label: String,
    /// Explicit parameter labels in declaration order.
    pub parameters: Vec<String>,
}

/// One named declaration with stable source spans and an owner-qualified identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    /// Source spelling of the declaration name.
    pub name: String,
    /// Owner-qualified identity used to avoid collisions across units.
    pub qualified_name: String,
    /// Declaration category.
    pub kind: SymbolKind,
    /// Full declaration span.
    pub full_span: SourceSpan,
    /// Tight name span used for editor selection.
    pub selection_span: SourceSpan,
    /// Source scope in which the declaration can be referenced.
    pub scope_span: SourceSpan,
    /// First byte offset at which sequential lookup can see the declaration.
    pub visible_from: usize,
    /// Cross-document visibility.
    pub visibility: SymbolVisibility,
    /// Optional declared named type used by editor member queries.
    pub type_name: Option<String>,
    /// Compact source-level declaration detail for hover and completion.
    pub detail: String,
    /// Explicit call shape when the declaration can be invoked.
    pub callable: Option<CallableSignature>,
    /// Nested declarations in source order.
    pub children: Vec<DocumentSymbol>,
}

/// Symbols extracted from one immutable document snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbols {
    owner: String,
    entries: Vec<DocumentSymbol>,
}

impl DocumentSymbols {
    /// Extracts a compilation-unit symbol with hierarchical declarations.
    #[must_use]
    pub fn from_snapshot(snapshot: &DocumentSnapshot) -> Self {
        let (owner, entries) = super::extract::extract(snapshot);
        Self { owner, entries }
    }

    pub(crate) fn from_editor_snapshot(snapshot: &DocumentSnapshot) -> Self {
        let mut symbols = Self::from_snapshot(snapshot);
        super::intrinsic_api::add_registry_symbols(snapshot, &mut symbols);
        symbols
    }

    /// Returns the program or unit owner name.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns root compilation-unit symbols in source order.
    #[must_use]
    pub fn entries(&self) -> &[DocumentSymbol] {
        &self.entries
    }

    pub(super) fn entries_mut(&mut self) -> &mut Vec<DocumentSymbol> {
        &mut self.entries
    }
}
