//! Readable source-local aliases that obey compiler namespace reservations.
//! See `docs/pascal/program-structure/units.md`.

use std::collections::HashSet;

use fpas_lexer::{Token, lex};

use crate::SymbolKind;
use crate::navigation::NavigationDocument;

/// Choose a deterministic unused alias without changing existing import spellings.
pub(super) fn new_import_alias(document: &NavigationDocument, unit: &str) -> String {
    let mut reserved = document
        .all_symbols()
        .into_iter()
        .filter(|symbol| {
            !matches!(
                symbol.kind,
                SymbolKind::Program
                    | SymbolKind::Unit
                    | SymbolKind::Field
                    | SymbolKind::Method
                    | SymbolKind::EnumMember
            )
        })
        .map(|symbol| symbol.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    reserved.extend(
        [
            "std",
            "integer",
            "real",
            "boolean",
            "string",
            "arraypopresult",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    reserved.extend(
        document
            .uses
            .iter()
            .filter_map(|import| import.unit.parts.first())
            .map(|name| name.to_ascii_lowercase()),
    );
    if matches!(
        document.snapshot.compilation_unit(),
        fpas_parser::CompilationUnit::Unit(_)
    ) {
        reserved.insert(
            document
                .owner
                .split('.')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase(),
        );
    }
    reserved.insert(unit.split('.').next().unwrap_or("").to_ascii_lowercase());
    let parts = unit.split('.').collect::<Vec<_>>();
    let available = |name: &str| {
        !reserved.contains(&name.to_ascii_lowercase())
            && matches!(
                lex(name).0.first().map(|token| &token.token),
                Some(Token::Ident(_))
            )
    };
    let mut candidate = parts.last().copied().unwrap_or("Imported").to_owned();
    if available(&candidate) {
        return candidate;
    }
    for part in parts.iter().rev().skip(1) {
        candidate = format!("{part}{candidate}");
        if available(&candidate) {
            return candidate;
        }
    }
    candidate.push_str("Unit");
    if available(&candidate) {
        return candidate;
    }
    for suffix in 2_u64.. {
        let name = format!("{candidate}{suffix}");
        if available(&name) {
            return name;
        }
    }
    unreachable!("a finite source cannot reserve every numbered alias")
}
