//! Source-visible imported record names for debugger construction metadata.
//! See `docs/pascal/program-structure/units.md`.

use fpas_unit::interface::InterfaceSymbol;

use super::InterfaceSet;

/// Checks that an exported symbol belongs to an explicitly imported interface.
pub(super) fn is_visible(interfaces: &InterfaceSet<'_>, symbol: &InterfaceSymbol) -> bool {
    interfaces.direct.iter().any(|interface| {
        interfaces.uses.iter().any(|import| {
            import
                .unit
                .parts
                .join(".")
                .eq_ignore_ascii_case(&interface.unit_name)
        }) && interface.symbols.iter().any(|candidate| {
            candidate
                .qualified_name
                .eq_ignore_ascii_case(&symbol.qualified_name)
        })
    })
}
