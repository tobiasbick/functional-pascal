//! Intrinsic nominal definitions needed by imported source-unit interfaces.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use crate::types::Ty;
use std::{collections::HashMap, sync::OnceLock};

/// Resolve canonical type metadata without introducing a source import or binding.
pub(crate) fn intrinsic_type(name: &str) -> Option<Ty> {
    static TYPES: OnceLock<HashMap<String, Ty>> = OnceLock::new();
    if !name.to_ascii_lowercase().starts_with("std.") {
        return None;
    }
    TYPES
        .get_or_init(|| {
            super::intrinsic_std_units()
                .iter()
                .flat_map(|unit| super::intrinsic_std_symbols(unit))
                .filter(|symbol| symbol.kind == super::IntrinsicStdSymbolKind::Type)
                .map(|symbol| (symbol.qualified_name.to_ascii_lowercase(), symbol.ty))
                .collect()
        })
        .get(&name.to_ascii_lowercase())
        .cloned()
}
