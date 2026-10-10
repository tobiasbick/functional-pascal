//! Record defaults and method bodies retain source-order value visibility.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::Checker;
use crate::types::Ty;
use fpas_parser::{RecordMethod, RecordType, TypeDef};
use std::collections::HashSet;

impl Checker {
    /// Check original AST expressions only after preceding value declarations are in scope.
    pub(crate) fn check_record_values(&mut self, definition: &TypeDef, record: &RecordType) {
        self.with_type_params(&definition.type_params, definition.span, |checker| {
            checker.check_record_values_in_scope(definition, record)
        });
    }

    fn check_record_values_in_scope(&mut self, definition: &TypeDef, record: &RecordType) {
        self.check_record_defaults(definition, record);
        let mut seen = HashSet::new();
        for method in &record.methods {
            if !seen.insert(method.name().to_ascii_lowercase()) {
                continue;
            }
            let name = format!("{}.{}", definition.name, method.name());
            let Some(symbol) = self.scopes.lookup_root(&name) else {
                continue;
            };
            let (params, result) = match &symbol.ty {
                Ty::Function(function) => {
                    (function.params.clone(), Some(*function.return_type.clone()))
                }
                Ty::Procedure(procedure) => (procedure.params.clone(), None),
                _ => continue,
            };
            let (type_params, source_params, body) = match method {
                RecordMethod::Function(routine) | RecordMethod::StaticFunction(routine) => {
                    (&routine.type_params, &routine.params, &routine.body)
                }
                RecordMethod::Procedure(routine) | RecordMethod::StaticProcedure(routine) => {
                    (&routine.type_params, &routine.params, &routine.body)
                }
            };
            let spans: Vec<_> = source_params.iter().map(|param| param.span).collect();
            self.check_method_body(&name, type_params, &params, &spans, result, body);
        }
    }
}
