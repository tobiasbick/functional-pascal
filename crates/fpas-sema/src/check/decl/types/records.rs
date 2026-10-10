//! Record structures and member signatures, checked before ordered bodies.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

mod bodies;
mod defaults;
mod fields;
mod methods;
mod signatures;

use super::Checker;
use crate::types::Ty;
use fpas_parser::{RecordType, TypeDef};
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    /// Collect record fields and defaults without checking executable expressions.
    pub(super) fn check_record_type_def(&mut self, definition: &TypeDef, record: &RecordType) {
        let fields = self.collect_record_fields(definition, record);
        self.register_record_defaults(definition, record);
        self.define_type_symbol(definition, Ty::Record(Arc::new(fields)));
    }

    /// Complete a record's member signatures after every structural header is resolved.
    pub(crate) fn collect_record_members(&mut self, definition: &TypeDef, record: &RecordType) {
        let Some(symbol) = self.scopes.lookup_type(&definition.name) else {
            return;
        };
        let mut ty = symbol.ty.clone();
        let Ty::Record(shape) = &ty else { return };
        let mut seen_members: HashSet<_> = shape
            .fields
            .iter()
            .map(|(name, _)| name.to_ascii_lowercase())
            .collect();
        let members =
            self.check_record_methods(&definition.name, &ty, &record.methods, &mut seen_members);
        if let Ty::Record(shape) = &mut ty {
            let shape = Arc::make_mut(shape);
            shape.methods = members.instance_methods;
            shape.static_functions = members.static_functions;
            shape.static_procedures = members.static_procedures;
        }
        self.define_type_symbol(definition, ty);
    }
}
