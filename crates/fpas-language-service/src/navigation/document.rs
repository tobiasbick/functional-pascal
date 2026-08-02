//! Query-ready view of one immutable source document.

use std::path::PathBuf;
use std::sync::Arc;

use fpas_lexer::{SpannedToken, lex};
use fpas_parser::CompilationUnit;

use crate::{DocumentSnapshot, DocumentSymbol, DocumentSymbols};

#[derive(Clone)]
pub(crate) struct NavigationDocument {
    pub(crate) path: PathBuf,
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) owner: String,
    pub(crate) uses: Vec<String>,
    pub(crate) roots: Vec<DocumentSymbol>,
    pub(crate) tokens: Vec<SpannedToken>,
}

impl NavigationDocument {
    pub(crate) fn new(snapshot: Arc<DocumentSnapshot>) -> Self {
        let symbols = DocumentSymbols::from_snapshot(&snapshot);
        let uses = match snapshot.compilation_unit() {
            CompilationUnit::Program(program) => &program.uses,
            CompilationUnit::Unit(unit) => &unit.uses,
        }
        .iter()
        .map(|used| used.parts.join("."))
        .collect();
        let tokens = lex(snapshot.source()).0;
        Self {
            path: snapshot.path().to_path_buf(),
            owner: symbols.owner().to_owned(),
            roots: symbols.entries().to_vec(),
            snapshot,
            uses,
            tokens,
        }
    }

    pub(crate) fn all_symbols(&self) -> Vec<&DocumentSymbol> {
        self.roots.iter().flat_map(all_symbols).collect()
    }

    pub(crate) fn top_level(&self) -> &[DocumentSymbol] {
        self.roots
            .first()
            .map(|root| root.children.as_slice())
            .unwrap_or_default()
    }

    pub(crate) fn uses_owner(&self, owner: &str) -> bool {
        self.owner.eq_ignore_ascii_case(owner)
            || self
                .uses
                .iter()
                .any(|used| used.eq_ignore_ascii_case(owner))
    }
}

fn all_symbols(symbol: &DocumentSymbol) -> Box<dyn Iterator<Item = &DocumentSymbol> + '_> {
    Box::new(std::iter::once(symbol).chain(symbol.children.iter().flat_map(all_symbols)))
}
