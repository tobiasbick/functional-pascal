//! Declaration symbols extracted from one parsed snapshot.

use fpas_diagnostics::SourceSpan;
use fpas_parser::{CompilationUnit, Decl};

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
    /// Function declaration.
    Function,
    /// Procedure declaration.
    Procedure,
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
}

/// Symbols extracted from one immutable document snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbols {
    owner: String,
    entries: Vec<DocumentSymbol>,
}

impl DocumentSymbols {
    /// Extracts compilation-unit and top-level declaration symbols.
    #[must_use]
    pub fn from_snapshot(snapshot: &DocumentSnapshot) -> Self {
        match snapshot.compilation_unit() {
            CompilationUnit::Program(program) => {
                let owner = program.name.clone();
                let mut entries = vec![DocumentSymbol {
                    name: program.name.clone(),
                    qualified_name: program.name.clone(),
                    kind: SymbolKind::Program,
                    full_span: program.span.into(),
                    selection_span: program.name_span.into(),
                }];
                entries.extend(
                    program
                        .declarations
                        .iter()
                        .map(|declaration| declaration_symbol(snapshot, &owner, declaration)),
                );
                Self { owner, entries }
            }
            CompilationUnit::Unit(unit) => {
                let owner = unit.name.parts.join(".");
                let mut entries = vec![DocumentSymbol {
                    name: owner.clone(),
                    qualified_name: owner.clone(),
                    kind: SymbolKind::Unit,
                    full_span: unit.span.into(),
                    selection_span: unit.name.span.into(),
                }];
                entries.extend(
                    unit.declarations
                        .iter()
                        .map(|declaration| declaration_symbol(snapshot, &owner, declaration)),
                );
                Self { owner, entries }
            }
        }
    }

    /// Returns the program or unit owner name.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns compilation-unit and top-level declaration symbols in source order.
    #[must_use]
    pub fn entries(&self) -> &[DocumentSymbol] {
        &self.entries
    }
}

fn declaration_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &Decl,
) -> DocumentSymbol {
    let (name, kind, span) = match declaration {
        Decl::Const(value) => (&value.name, SymbolKind::Constant, value.span),
        Decl::Var(value) => (&value.name, SymbolKind::Variable, value.span),
        Decl::MutableVar(value) => (&value.name, SymbolKind::MutableVariable, value.span),
        Decl::TypeDef(value) => (&value.name, SymbolKind::Type, value.span),
        Decl::Function(value) => (&value.name, SymbolKind::Function, value.span),
        Decl::Procedure(value) => (&value.name, SymbolKind::Procedure, value.span),
    };
    let full_span = span.into();
    DocumentSymbol {
        name: name.clone(),
        qualified_name: format!("{owner}.{name}"),
        kind,
        selection_span: name_span(snapshot, full_span, name),
        full_span,
    }
}

fn name_span(snapshot: &DocumentSnapshot, full_span: SourceSpan, name: &str) -> SourceSpan {
    let source = snapshot.source();
    let end = full_span
        .offset
        .saturating_add(full_span.length)
        .min(source.len());
    let Some(fragment) = source.get(full_span.offset..end) else {
        return full_span;
    };
    let Some(relative_offset) = fragment
        .to_ascii_lowercase()
        .find(&name.to_ascii_lowercase())
    else {
        return full_span;
    };
    let offset = full_span.offset + relative_offset;
    let Some(position) = snapshot.line_index().position(source, offset) else {
        return full_span;
    };
    SourceSpan::new_with_source(
        offset,
        name.len(),
        u32::try_from(position.line + 1).unwrap_or(u32::MAX),
        u32::try_from(position.byte_column + 1).unwrap_or(u32::MAX),
        full_span.source_id,
    )
}
