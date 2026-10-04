//! Whole-unit type headers, resolved before declaration bodies.
//!
//! **Documentation:** `docs/pascal/language/types/README.md`.

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_UNKNOWN_TYPE};
use fpas_parser::{Decl, TypeBody, TypeDef};
use std::collections::{HashMap, HashSet};

/// Pending type bodies and recursion state for whole-unit header resolution.
#[derive(Default)]
pub(in crate::check) struct TypeCollection {
    pending: HashMap<String, TypeDef>,
    resolving: Vec<(String, bool)>,
    accepted: HashSet<(u32, usize)>,
    dictionary_keys: Vec<(Ty, fpas_lexer::Span)>,
    /// Whether type bodies are being resolved before ordered declarations.
    pub(in crate::check) collecting: bool,
}

impl Checker {
    /// Collect every unit-level type name before resolving any type body.
    pub(crate) fn collect_unit_types(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            let name = definition.name.to_ascii_lowercase();
            if !self.scopes.define(
                &definition.name,
                Symbol {
                    ty: Ty::Named(definition.name.clone()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            ) {
                self.error_with_code(SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate type `{}`", definition.name),
                    "A type, value, routine, and import alias must have distinct names in the same scope.", definition.span);
                continue;
            }
            self.type_collection
                .accepted
                .insert((definition.span.source_id, definition.span.offset));
            self.type_collection
                .pending
                .insert(name, definition.clone());
        }
        let previous = self.type_collection.collecting;
        self.type_collection.collecting = true;
        for declaration in declarations {
            if let Decl::TypeDef(definition) = declaration {
                self.resolve_type_header(&definition.name.to_ascii_lowercase());
            }
        }
        self.type_collection.collecting = previous;
        if !previous {
            for (key, span) in std::mem::take(&mut self.type_collection.dictionary_keys) {
                self.check_dictionary_key_type(&key, span);
            }
        }
        self.validate_finite_type_headers(declarations);
    }

    /// Resolve a collected type lazily, preserving nominal recursion and rejecting alias cycles.
    pub(crate) fn resolve_type_header(&mut self, name: &str) {
        let canonical = name.to_ascii_lowercase();
        if let Some(index) = self
            .type_collection
            .resolving
            .iter()
            .position(|(name, _)| name == &canonical)
        {
            if self.type_collection.resolving[index..]
                .iter()
                .all(|(_, alias)| *alias)
            {
                let definition = &self.type_collection.pending[&canonical];
                self.error_with_code(
                    SEMA_UNKNOWN_TYPE,
                    format!("Cyclic type alias `{name}`"),
                    "End the alias chain at a concrete type, for example `type Name = integer;`.",
                    definition.span,
                );
            }
            return;
        }
        let Some(definition) = self.type_collection.pending.get(&canonical).cloned() else {
            return;
        };
        self.type_collection.resolving.push((
            canonical.clone(),
            matches!(definition.body, TypeBody::Alias(_)),
        ));
        let outer_scopes = self.scopes.suspend_nested_scopes();
        self.check_type_def(&definition);
        self.scopes.restore_nested_scopes(outer_scopes);
        self.type_collection.resolving.pop();
        self.type_collection.pending.remove(&canonical);
    }

    /// Check whether this exact declaration owns a previously collected header.
    pub(in crate::check) fn has_collected_type(&self, definition: &TypeDef) -> bool {
        self.type_collection
            .accepted
            .contains(&(definition.span.source_id, definition.span.offset))
    }
}

impl TypeCollection {
    /// Delay component checks until all forward nominal declarations are available.
    pub(in crate::check) fn defer_dictionary_key(
        &mut self,
        key: &Ty,
        span: fpas_lexer::Span,
    ) -> bool {
        if !self.collecting {
            return false;
        }
        self.dictionary_keys.push((key.clone(), span));
        true
    }

    /// Expose parameter declarations while a recursive nominal header is pending.
    pub(in crate::check) fn pending_type_params(
        &self,
        name: &str,
    ) -> Option<Vec<crate::types::GenericParamDef>> {
        self.pending
            .get(&name.to_ascii_lowercase())
            .map(|definition| {
                definition
                    .type_params
                    .iter()
                    .map(|parameter| crate::types::GenericParamDef {
                        name: parameter.name.clone(),
                        constraint: parameter
                            .constraint
                            .as_deref()
                            .and_then(crate::types::TypeConstraint::from_name),
                    })
                    .collect()
            })
    }
}
