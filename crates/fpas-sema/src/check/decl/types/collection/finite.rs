//! Finite construction of recursive records and enum alternatives.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NO_FINITE_TYPE_VALUE;
use fpas_parser::Decl;
use std::collections::HashSet;

impl Checker {
    /// Reject mandatory recursive payloads after all nominal definitions are available.
    pub(super) fn validate_finite_types(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            if !self.has_collected_type(definition) {
                continue;
            }
            let Some(symbol) = self.scopes.lookup_type(&definition.name) else {
                continue;
            };
            let mut cycle = Vec::new();
            if !self.type_has_finite_value(
                &symbol.ty,
                &mut HashSet::new(),
                &mut Vec::new(),
                &mut cycle,
            ) && !cycle.is_empty()
            {
                self.error_with_code(
                    SEMA_NO_FINITE_TYPE_VALUE,
                    format!("Type `{}` has no type finite value: {}", definition.name, cycle.join(" -> ")),
                    "Break the mandatory cycle with Option, an empty container, or an enum alternative whose required payloads allow finite construction.",
                    definition.span,
                );
            }
        }
    }

    fn type_has_finite_value(
        &self,
        ty: &Ty,
        active: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycle: &mut Vec<String>,
    ) -> bool {
        // A finite alternative must not leave its optional cycle as the failure witness.
        let prior_cycle_length = cycle.len();
        let resolved = self.resolve_visible_type(ty);
        let (name, fields, variants) = match &resolved {
            Ty::Record(record) => (&record.name, Some(&record.fields), None),
            Ty::Enum(enumeration) => (&enumeration.name, None, Some(&enumeration.variants)),
            Ty::Result(ok, error) => {
                let finite = self.type_has_finite_value(ok, active, path, cycle)
                    || self.type_has_finite_value(error, active, path, cycle);
                if finite {
                    cycle.truncate(prior_cycle_length);
                }
                return finite;
            }
            // Containers can be empty, callables store an environment rather than their
            // signature's values, and task handles do not inline their output value.
            _ => return true,
        };
        let key = name.to_ascii_lowercase();
        if !active.insert(key.clone()) {
            if cycle.is_empty() {
                *cycle = path.clone();
                cycle.push(name.clone());
            }
            return false;
        }
        let finite = if let Some(fields) = fields {
            fields.iter().all(|(field, ty)| {
                path.push(format!("{name}.{field}"));
                let finite = self.type_has_finite_value(ty, active, path, cycle);
                path.pop();
                finite
            })
        } else {
            variants.unwrap().iter().any(|variant| {
                variant.fields.iter().all(|(field, ty)| {
                    path.push(format!("{name}.{}.{field}", variant.name));
                    let finite = self.type_has_finite_value(ty, active, path, cycle);
                    path.pop();
                    finite
                })
            })
        };
        active.remove(&key);
        if finite {
            cycle.truncate(prior_cycle_length);
        }
        finite
    }
}
