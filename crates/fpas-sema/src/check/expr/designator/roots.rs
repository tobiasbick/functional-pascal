//! Shared storage-root resolution for typed paths and assignment permissions.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_parser::DesignatorPart;

impl Checker {
    /// Resolve the longest value prefix, retaining its mutability and symbol kind.
    ///
    /// Import qualifiers identify namespaces; the binding after the qualifier
    /// determines whether fields and collection elements may be assigned.
    pub(crate) fn designator_root_symbol(
        &self,
        parts: &[DesignatorPart],
    ) -> Option<(&Symbol, usize)> {
        let mut qualified = String::new();
        let mut resolved = None;
        for (index, part) in parts.iter().enumerate() {
            let DesignatorPart::Ident(name, _) = part else {
                break;
            };
            if !qualified.is_empty() {
                qualified.push('.');
            }
            qualified.push_str(name);
            if let Some(symbol) = self.scopes.lookup(&self.qualified_import_name(&qualified))
                && matches!(
                    symbol.kind,
                    SymbolKind::Const
                        | SymbolKind::Var
                        | SymbolKind::Param
                        | SymbolKind::VarParam
                        | SymbolKind::ForVar
                )
            {
                resolved = Some((symbol, index + 1));
            }
        }
        resolved
    }
}
