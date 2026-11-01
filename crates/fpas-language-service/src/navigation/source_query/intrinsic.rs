//! Intrinsic declaration facts come from the compiler registry, never API stub bodies.

use crate::{DocumentSymbol, SymbolKind};
use fpas_sema::{IntrinsicStdSymbol, IntrinsicStdSymbolKind, Ty};

pub(super) fn lookup(qualified: &str) -> Option<IntrinsicStdSymbol> {
    let unit = fpas_sema::intrinsic_std_units()
        .iter()
        .filter(|unit| {
            qualified.len() > unit.len()
                && qualified
                    .get(..unit.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(unit))
                && qualified.as_bytes().get(unit.len()) == Some(&b'.')
        })
        .max_by_key(|unit| unit.len())?;
    Catalog::new(unit).lookup(qualified)
}

/// Shared compiler-owned signature for inspection and hover; API bodies carry no type evidence.
pub(in crate::navigation) fn detail(qualified: &str) -> Option<String> {
    let symbol = lookup(qualified)?;
    let name = qualified.rsplit('.').next()?;
    let (params, result) = match &symbol.ty {
        Ty::Function(f) => (&f.params, Some(f.return_type.as_ref())),
        Ty::Procedure(p) => (&p.params, None),
        _ => return None,
    };
    Some(format!(
        "{} {name}({}){}",
        if result.is_some() {
            "function"
        } else {
            "procedure"
        },
        params
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; "),
        result.map_or_else(String::new, |ty| format!(": {ty}"))
    ))
}

pub(super) struct Catalog {
    unit: String,
    symbols: Vec<IntrinsicStdSymbol>,
}

impl Catalog {
    pub(super) fn new(unit: &str) -> Self {
        Self {
            unit: unit.into(),
            symbols: fpas_sema::intrinsic_std_symbols(unit),
        }
    }
    pub(super) fn known(&self, symbol: &DocumentSymbol) -> bool {
        if symbol.kind == SymbolKind::Unit {
            return !self.symbols.is_empty()
                && self.unit.eq_ignore_ascii_case(&symbol.qualified_name);
        }
        self.lookup(&symbol.qualified_name).is_some()
    }
    fn lookup(&self, qualified: &str) -> Option<IntrinsicStdSymbol> {
        if let Some(symbol) = self
            .symbols
            .iter()
            .find(|s| s.qualified_name.eq_ignore_ascii_case(qualified))
        {
            return Some(symbol.clone());
        }
        // Record fields are part of the registered nominal type, not global registry entries.
        let (owner, field) = qualified.rsplit_once('.')?;
        let parent = self
            .symbols
            .iter()
            .find(|s| s.qualified_name.eq_ignore_ascii_case(owner))?;
        let Ty::Record(record) = &parent.ty else {
            return None;
        };
        let (_, ty) = record
            .fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(field))?;
        Some(IntrinsicStdSymbol {
            qualified_name: qualified.into(),
            kind: IntrinsicStdSymbolKind::Constant,
            ty: ty.clone(),
        })
    }
}

pub(super) fn kind(kind: IntrinsicStdSymbolKind) -> SymbolKind {
    match kind {
        IntrinsicStdSymbolKind::Constant => SymbolKind::Constant,
        IntrinsicStdSymbolKind::Function => SymbolKind::Function,
        IntrinsicStdSymbolKind::Procedure => SymbolKind::Procedure,
        IntrinsicStdSymbolKind::Type => SymbolKind::Type,
        IntrinsicStdSymbolKind::EnumMember => SymbolKind::EnumMember,
    }
}
