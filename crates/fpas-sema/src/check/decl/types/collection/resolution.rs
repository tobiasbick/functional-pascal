//! Lazy structural resolution with nominal recursion and explicit alias-cycle errors.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_CYCLIC_TYPE_ALIAS;
use fpas_parser::TypeBody;

impl Checker {
    /// Resolve a pending header, returning an override only for an active recursive reference.
    pub(crate) fn resolve_collected_type(&mut self, name: &str) -> Option<Ty> {
        let key = canonical_symbol_name(name);
        if let Some(index) = self
            .type_collection
            .resolving
            .iter()
            .rposition(|(name, _)| name == &key)
        {
            if self.type_collection.resolving[index..]
                .iter()
                .all(|(_, alias)| *alias)
            {
                if self.type_collection.reported_aliases.insert(key.clone()) {
                    let definition = self.type_collection.pending[&key].clone();
                    let mut cycle = self.type_collection.resolving[index..]
                        .iter()
                        .map(|(name, _)| self.type_collection.pending[name].name.clone())
                        .collect::<Vec<_>>();
                    cycle.push(definition.name.clone());
                    self.error_with_code(
                        SEMA_CYCLIC_TYPE_ALIAS,
                        format!("Cyclic type alias: {}", cycle.join(" -> ")),
                        "End the alias chain at a concrete type. Use a record or enum for a finite recursive value.",
                        definition.span,
                    );
                }
                return Some(Ty::Error);
            }
            // Expand aliases inside a nominal cycle so the stored back-reference names
            // the nominal type, rather than a partly resolved container alias.
            if let TypeBody::Alias(expression) = self.type_collection.pending[&key].body.clone() {
                self.type_collection.resolving.push((key, true));
                let ty = self.resolve_type_expr(&expression);
                self.type_collection.resolving.pop();
                return Some(ty);
            }
            return None;
        }
        let Some(definition) = self.type_collection.pending.get(&key).cloned() else {
            return None;
        };
        self.type_collection
            .resolving
            .push((key.clone(), matches!(definition.body, TypeBody::Alias(_))));
        self.check_type_def(&definition);
        self.type_collection.resolving.pop();
        self.type_collection.pending.remove(&key);
        None
    }
}
