//! Whole-unit type collection, separate from ordered value and body checking.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

mod finite;
mod generic_recursion;
mod resolution;

use super::Checker;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{Decl, TypeBody, TypeDef};
use std::collections::{HashMap, HashSet};

/// Type headers retained only during structural resolution, never for expression checking.
#[derive(Default)]
pub(crate) struct TypeCollection {
    pending: HashMap<String, TypeDef>,
    resolving: Vec<(String, bool)>,
    accepted: HashSet<(u32, usize)>,
    reported_aliases: HashSet<String>,
    value_positions: HashMap<String, usize>,
    /// Whether only structural definitions are being resolved.
    pub(crate) collecting: bool,
}

impl Checker {
    /// Resolve every type and member signature before checking declarations in source order.
    pub(crate) fn collect_unit_types(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            let (name, offset) = match declaration {
                Decl::Const(value) => (&value.name, value.span.offset),
                Decl::Var(value) => (&value.name, value.span.offset),
                Decl::Function(routine) => (&routine.name, routine.span.offset),
                Decl::Procedure(routine) => (&routine.name, routine.span.offset),
                Decl::TypeDef(_) => continue,
            };
            self.type_collection
                .value_positions
                .entry(canonical_symbol_name(name))
                .or_insert(offset);
        }
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            if self.scopes.imports.is_alias(&definition.name) {
                continue;
            }
            if !self.scopes.define_with_declaration(
                &definition.name,
                Symbol {
                    constant: None,
                    ty: if definition.type_params.is_empty() {
                        Ty::Named(definition.name.clone())
                    } else if matches!(definition.body, TypeBody::Enum(_)) {
                        Ty::Enum(std::sync::Arc::new(crate::types::EnumTy::header(
                            definition.name.clone(),
                            Self::resolve_type_params(&definition.type_params),
                        )))
                    } else {
                        Ty::Record(std::sync::Arc::new(crate::types::RecordTy::header(
                            definition.name.clone(),
                            self.scopes
                                .function_ctx
                                .as_ref()
                                .and_then(|context| context.owner_unit.clone()),
                            Self::resolve_type_params(&definition.type_params),
                        )))
                    },
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
                definition.span,
            ) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate type `{}`", definition.name),
                    "Each declaration name must be unique in the same scope.",
                    definition.span,
                );
                continue;
            }
            self.type_collection
                .accepted
                .insert((definition.span.source_id, definition.span.offset));
            self.type_collection
                .pending
                .insert(canonical_symbol_name(&definition.name), definition.clone());
        }
        self.type_collection.collecting = true;
        for declaration in declarations {
            if let Decl::TypeDef(definition) = declaration {
                self.resolve_collected_type(&definition.name);
            }
        }
        self.type_collection.collecting = false;
        self.refresh_collected_aliases(declarations);
        // All field/payload definitions are complete before method-level generic scopes exist.
        for declaration in declarations {
            if let Decl::TypeDef(definition) = declaration
                && self.has_collected_type(definition)
                && let TypeBody::Record(record) = &definition.body
            {
                self.collect_record_members(definition, record);
            }
        }
        self.refresh_collected_aliases(declarations);
        for declaration in declarations {
            if let Decl::TypeDef(definition) = declaration
                && self.has_collected_type(definition)
                && matches!(definition.body, TypeBody::Alias(_))
                && let Some(symbol) = self.scopes.lookup_type(&definition.name)
                && let Ty::Enum(enumeration) = symbol.ty.clone()
            {
                self.register_enum_alias_variant_symbols(definition, &enumeration);
            }
        }
        self.validate_generic_recursion(declarations);
        self.validate_finite_types(declarations);
    }

    // Recursive aliases can retain a nominal placeholder or a record's pre-member shape.
    fn refresh_collected_aliases(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            if let Decl::TypeDef(definition) = declaration
                && self.has_collected_type(definition)
                && matches!(definition.body, TypeBody::Alias(_))
                && let Some(symbol) = self.scopes.lookup_type(&definition.name)
            {
                let ty = self.resolve_visible_type(&symbol.ty);
                self.define_type_symbol(definition, ty);
            }
        }
    }

    /// Whether this source declaration owns a successfully collected type name.
    pub(crate) fn has_collected_type(&self, definition: &TypeDef) -> bool {
        self.type_collection
            .accepted
            .contains(&(definition.span.source_id, definition.span.offset))
    }

    /// Preserve a preceding value's priority over an enum's optional short variant alias.
    pub(in crate::check::decl::types) fn has_preceding_value(
        &self,
        name: &str,
        offset: usize,
    ) -> bool {
        self.type_collection
            .value_positions
            .get(&canonical_symbol_name(name))
            .is_some_and(|position| *position < offset)
    }
}
