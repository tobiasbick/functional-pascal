//! Symbol definitions, imports, signatures, and object-local references.

/// Category of a link-time symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolKind {
    /// Register-bytecode function.
    Function,
    /// Dense global slot.
    Global,
    /// Record layout.
    Record,
    /// Enum layout.
    Enum,
}

/// Object-local table entry supplied by a definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefinitionTarget {
    /// Function-table index.
    Function(u32),
    /// Global-table index.
    Global(u32),
    /// Record-layout index.
    Record(u32),
    /// Enum-layout index.
    Enum(u32),
}

impl DefinitionTarget {
    /// Return the symbol category represented by this target.
    #[must_use]
    pub const fn kind(self) -> SymbolKind {
        match self {
            Self::Function(_) => SymbolKind::Function,
            Self::Global(_) => SymbolKind::Global,
            Self::Record(_) => SymbolKind::Record,
            Self::Enum(_) => SymbolKind::Enum,
        }
    }

    /// Return the object-local table index.
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Function(index)
            | Self::Global(index)
            | Self::Record(index)
            | Self::Enum(index) => index,
        }
    }
}

/// Public or private symbol supplied by one object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectDefinition {
    /// Canonical fully qualified symbol name.
    pub name: String,
    /// Object-local table entry implementing the symbol.
    pub target: DefinitionTarget,
    /// Whether another object may import this definition.
    pub public: bool,
}

/// Compatibility contract required by an imported symbol.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImportShape {
    /// Callable ABI required by a direct reference.
    Function {
        /// Positional argument count.
        arity: u8,
        /// Captured register count.
        capture_count: u16,
        /// Whether the callable returns a value.
        returns_value: bool,
    },
    /// Global mutability contract.
    Global {
        /// Whether imported code may store after initialization.
        mutable: bool,
    },
    /// Ordered record field names.
    Record {
        /// Canonical field names in declaration order.
        fields: Vec<String>,
    },
    /// Ordered variants and their associated field names.
    Enum {
        /// Canonical variant name followed by canonical associated fields.
        variants: Vec<(String, Vec<String>)>,
    },
}

impl ImportShape {
    /// Return the required symbol category.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        match self {
            Self::Function { .. } => SymbolKind::Function,
            Self::Global { .. } => SymbolKind::Global,
            Self::Record { .. } => SymbolKind::Record,
            Self::Enum { .. } => SymbolKind::Enum,
        }
    }
}

/// One public definition required from another object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectImport {
    /// Canonical fully qualified symbol name.
    pub name: String,
    /// Required ABI or layout shape.
    pub shape: ImportShape,
}

/// Link target used by relocatable code or constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SymbolReference {
    /// Object-local table index.
    Local(u32),
    /// Object-local import-table index.
    Import(u32),
}
