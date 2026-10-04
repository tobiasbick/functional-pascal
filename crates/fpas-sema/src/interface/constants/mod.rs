//! Static constant values from lexical bindings, compiled units and standard units.
//!
//! **Documentation:** `docs/pascal/language/types/records.md` (Default field values).

use std::collections::HashMap;

mod aggregates;
mod conversion;
mod evaluation;
mod operators;
mod static_data;

use conversion::from_interface;
pub(crate) use conversion::{to_interface, to_scalar};
pub(crate) use evaluation::{StaticEvaluationError, StaticRecord};
use fpas_bytecode::Value;
use fpas_unit::interface::{SymbolKind, UnitInterface};

/// Static value data and scalar interface constants available during checking.
#[derive(Default)]
pub(crate) struct StaticConstants {
    values: HashMap<String, Value>,
    bindings: HashMap<(u32, usize), Value>,
}

impl StaticConstants {
    /// Store evaluated static data under its lexical declaration identity.
    pub(crate) fn insert_binding(&mut self, declaration: fpas_lexer::Span, value: Option<Value>) {
        if let Some(value) = value {
            self.bindings
                .insert((declaration.source_id, declaration.offset), value);
        }
    }

    /// Resolve static data using the checked declaration rather than a shadowed name.
    pub(crate) fn binding_value(
        &self,
        name: &str,
        declaration: Option<fpas_lexer::Span>,
    ) -> Option<Value> {
        if let Some(declaration) = declaration {
            self.bindings
                .get(&(declaration.source_id, declaration.offset))
                .cloned()
        } else {
            self.values
                .get(&name.to_ascii_lowercase())
                .cloned()
                .or_else(|| fpas_std::intrinsic_std_constant_value(name))
        }
    }

    /// Install static scalar and aggregate data supplied by compiled-unit interfaces.
    pub(crate) fn install(&mut self, interfaces: &[UnitInterface]) {
        for interface in interfaces {
            for symbol in &interface.symbols {
                let value = match &symbol.kind {
                    SymbolKind::Constant(Some(value)) => {
                        from_interface(value).map(conversion::from_scalar)
                    }
                    SymbolKind::AggregateConstant(value) => static_data::restore(value),
                    _ => None,
                };
                if let Some(value) = value {
                    self.values
                        .insert(symbol.qualified_name.to_ascii_lowercase(), value);
                }
            }
        }
    }
    /// Persist checked data while keeping aggregate values in immutable global storage.
    pub(super) fn binding_interface_kind(&self, declaration: fpas_lexer::Span) -> SymbolKind {
        let Some(value) = self
            .bindings
            .get(&(declaration.source_id, declaration.offset))
        else {
            return SymbolKind::Constant(None);
        };
        if let Some(value) = to_scalar(value).and_then(to_interface) {
            SymbolKind::Constant(Some(value))
        } else if let Some(value) = static_data::persist(value) {
            SymbolKind::AggregateConstant(value)
        } else {
            SymbolKind::Constant(None)
        }
    }
}
