//! Read-only intrinsic standard-library declarations for editor tooling.
//!
//! **Documentation:** `docs/pascal/std/README.md`

use crate::check::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;

/// Returns every standard-library unit implemented as compiler or runtime intrinsics.
#[must_use]
pub fn intrinsic_std_units() -> &'static [&'static str] {
    fpas_std::STD_UNITS_INTRINSIC
}

/// Editor-facing category of an intrinsic standard-library declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicStdSymbolKind {
    /// Compile-time constant.
    Constant,
    /// Function with a fixed or polymorphic signature.
    Function,
    /// Procedure with a fixed or polymorphic signature.
    Procedure,
    /// Record or enum type.
    Type,
    /// Enum member.
    EnumMember,
}

/// One intrinsic `Std.*` declaration exactly as registered by semantic analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicStdSymbol {
    /// Fully qualified, case-preserving declaration name.
    pub qualified_name: String,
    /// Semantic declaration category.
    pub kind: IntrinsicStdSymbolKind,
    /// Registered semantic type or polymorphic placeholder.
    pub ty: Ty,
}

/// Returns the declarations registered for one intrinsic standard-library unit.
///
/// Source-defined units return an empty list. Callers should pass names from
/// [`fpas_std::STD_UNITS_INTRINSIC`].
#[must_use]
pub fn intrinsic_std_symbols(unit: &str) -> Vec<IntrinsicStdSymbol> {
    if !fpas_std::STD_UNITS_INTRINSIC
        .iter()
        .any(|known| known.eq_ignore_ascii_case(unit))
    {
        return Vec::new();
    }

    let mut checker = Checker::new();
    super::register_single_std_unit(&mut checker, unit);
    let prefix = format!("{unit}.");
    checker
        .scopes
        .root_symbols_with_prefix(&prefix)
        .into_iter()
        .filter_map(|(qualified_name, symbol)| {
            symbol_kind(symbol.kind).map(|kind| IntrinsicStdSymbol {
                qualified_name,
                kind,
                ty: symbol.ty,
            })
        })
        .collect()
}

fn symbol_kind(kind: SymbolKind) -> Option<IntrinsicStdSymbolKind> {
    match kind {
        SymbolKind::Const => Some(IntrinsicStdSymbolKind::Constant),
        SymbolKind::Function | SymbolKind::BuiltinStd => Some(IntrinsicStdSymbolKind::Function),
        SymbolKind::Procedure => Some(IntrinsicStdSymbolKind::Procedure),
        SymbolKind::Type => Some(IntrinsicStdSymbolKind::Type),
        SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor => {
            Some(IntrinsicStdSymbolKind::EnumMember)
        }
        SymbolKind::Var | SymbolKind::Param | SymbolKind::ForVar => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{IntrinsicStdSymbolKind, intrinsic_std_symbols};

    #[test]
    fn intrinsic_std_symbols_returns_registered_fs_signatures() {
        let symbols = intrinsic_std_symbols(fpas_std::STD_UNIT_FS);
        let read_text = symbols
            .iter()
            .find(|symbol| symbol.qualified_name == "Std.Fs.ReadText")
            .expect("Std.Fs.ReadText declaration");

        assert_eq!(read_text.kind, IntrinsicStdSymbolKind::Function);
        assert_eq!(
            read_text.ty.to_string(),
            "function(Path: string): Result of string, string"
        );
    }

    #[test]
    fn intrinsic_std_symbols_excludes_source_units() {
        assert!(intrinsic_std_symbols(fpas_std::STD_UNIT_TUI).is_empty());
    }

    #[test]
    fn intrinsic_std_symbols_cover_every_registered_public_name() {
        for unit in fpas_std::STD_UNITS_INTRINSIC {
            let symbols = intrinsic_std_symbols(unit);
            for name in fpas_std::std_unit_symbols(unit) {
                assert!(
                    symbols
                        .iter()
                        .any(|symbol| symbol.qualified_name.eq_ignore_ascii_case(name)),
                    "{name} is absent from the {unit} editor catalog"
                );
            }
        }
    }
}
