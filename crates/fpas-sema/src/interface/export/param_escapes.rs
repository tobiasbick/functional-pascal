//! Export inferred parameter escapes of public routines and record routines.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use std::collections::HashMap;

use fpas_parser::{Decl, TypeBody, Unit, Visibility};
use fpas_unit::interface::RoutineParamEscapes;

use crate::scope::canonical_symbol_name;

/// List every public routine, so importers can tell routines from callable values.
pub(super) fn export_param_escapes(
    unit: &Unit,
    inferred: &HashMap<String, Vec<bool>>,
) -> Vec<RoutineParamEscapes> {
    let unit_name = unit.name.parts.join(".");
    let mut exported = Vec::new();
    let mut push = |local: String| {
        let escapes = inferred
            .get(&canonical_symbol_name(&local))
            .filter(|escapes| escapes.contains(&true))
            .cloned()
            .unwrap_or_default();
        exported.push(RoutineParamEscapes {
            qualified_name: format!("{unit_name}.{local}"),
            escapes,
        });
    };
    for declaration in &unit.declarations {
        if declaration.visibility() != Visibility::Public {
            continue;
        }
        match declaration {
            Decl::Function(function) => push(function.name.clone()),
            Decl::Procedure(procedure) => push(procedure.name.clone()),
            Decl::TypeDef(definition) => {
                let TypeBody::Record(record) = &definition.body else {
                    continue;
                };
                for method in &record.methods {
                    if method.visibility() == Visibility::Public {
                        push(format!("{}.{}", definition.name, method.name()));
                    }
                }
            }
            Decl::Const(_) | Decl::Var(_) => {}
        }
    }
    exported
}
