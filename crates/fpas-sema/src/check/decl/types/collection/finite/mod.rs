//! Finite construction of recursive records and enum alternatives.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

mod graph;

use super::Checker;
use fpas_diagnostics::codes::SEMA_NO_FINITE_TYPE_VALUE;
use fpas_parser::Decl;
use graph::FiniteGraph;

impl Checker {
    /// Reject mandatory recursive payloads after solving the shared construction graph.
    /// See `docs/pascal/language/types/declaration-order.md`.
    pub(super) fn validate_finite_types(&mut self, declarations: &[Decl]) {
        let definitions: Vec<_> = declarations
            .iter()
            .filter_map(|declaration| {
                let Decl::TypeDef(definition) = declaration else {
                    return None;
                };
                if !self.has_collected_type(definition) {
                    return None;
                }
                let symbol = self.scopes.lookup_type(&definition.name)?;
                Some((definition, symbol.ty.clone()))
            })
            .collect();
        let (graph, roots) = FiniteGraph::new(self, definitions.iter().map(|(_, ty)| ty));
        let values = graph.solve();
        for ((definition, _), root) in definitions.iter().zip(roots) {
            if values.is_finite(root) {
                continue;
            }
            let cycle = graph.cycle(root, &values);
            if !cycle.is_empty() {
                self.error_with_code(
                    SEMA_NO_FINITE_TYPE_VALUE,
                    format!("Type `{}` has no type finite value: {}", definition.name, cycle.join(" -> ")),
                    "Break the mandatory cycle with Option, an empty container, or an enum alternative whose required payloads allow finite construction.",
                    definition.span,
                );
            }
        }
    }
}
