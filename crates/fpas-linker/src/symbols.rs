//! Canonical symbol collection, visibility, and ABI/layout validation.

use std::collections::BTreeMap;

use fpas_unit::object::{
    DefinitionTarget, ImportShape, ObjectDefinition, RelocatableObject, SymbolKind, SymbolReference,
};

use crate::RegisterLinkError;

#[derive(Debug, Clone, Copy)]
pub(super) struct ResolvedTarget {
    pub object: usize,
    pub target: DefinitionTarget,
}

pub(super) struct SymbolTable {
    definitions: BTreeMap<String, (usize, usize)>,
    imports: Vec<Vec<ResolvedTarget>>,
}

impl SymbolTable {
    pub(super) fn build(objects: &[&RelocatableObject]) -> Result<Self, RegisterLinkError> {
        let mut definitions = BTreeMap::new();
        for (object_index, object) in objects.iter().enumerate() {
            for (definition_index, definition) in object.definitions.iter().enumerate() {
                if let Some(&(existing_object, existing_definition)) =
                    definitions.get(&definition.name)
                {
                    if !matching_layout_definition(
                        objects,
                        (existing_object, existing_definition),
                        (object_index, definition_index),
                    ) {
                        return Err(RegisterLinkError::DuplicateDefinition(
                            definition.name.clone(),
                        ));
                    }
                    let existing = &objects[existing_object].definitions[existing_definition];
                    if definition.public && !existing.public {
                        definitions
                            .insert(definition.name.clone(), (object_index, definition_index));
                    }
                } else {
                    definitions.insert(definition.name.clone(), (object_index, definition_index));
                }
            }
        }
        let mut imports = Vec::with_capacity(objects.len());
        for (object_index, object) in objects.iter().enumerate() {
            let mut resolved = Vec::with_capacity(object.imports.len());
            for import in &object.imports {
                let Some(&(owner_index, definition_index)) = definitions.get(&import.name) else {
                    return Err(RegisterLinkError::UnresolvedImport {
                        owner: object.owner.clone(),
                        name: import.name.clone(),
                        kind: import.shape.kind(),
                    });
                };
                let owner = objects[owner_index];
                let definition = &owner.definitions[definition_index];
                if owner_index == object_index || !definition.public {
                    return Err(RegisterLinkError::PrivateImport {
                        owner: object.owner.clone(),
                        name: import.name.clone(),
                    });
                }
                if definition.target.kind() != import.shape.kind() {
                    return Err(RegisterLinkError::ImportKind {
                        owner: object.owner.clone(),
                        name: import.name.clone(),
                        expected: import.shape.kind(),
                        actual: definition.target.kind(),
                    });
                }
                validate_shape(owner, definition, &import.shape).map_err(|detail| {
                    RegisterLinkError::IncompatibleImport {
                        owner: object.owner.clone(),
                        name: import.name.clone(),
                        detail,
                    }
                })?;
                resolved.push(ResolvedTarget {
                    object: owner_index,
                    target: definition.target,
                });
            }
            imports.push(resolved);
        }
        Ok(Self {
            definitions,
            imports,
        })
    }

    pub(super) fn resolve(
        &self,
        object: usize,
        reference: SymbolReference,
        kind: SymbolKind,
    ) -> Result<ResolvedTarget, RegisterLinkError> {
        let resolved = match reference {
            SymbolReference::Local(index) => ResolvedTarget {
                object,
                target: match kind {
                    SymbolKind::Function => DefinitionTarget::Function(index),
                    SymbolKind::Global => DefinitionTarget::Global(index),
                    SymbolKind::Record => DefinitionTarget::Record(index),
                    SymbolKind::Enum => DefinitionTarget::Enum(index),
                },
            },
            SymbolReference::Import(index) => *self
                .imports
                .get(object)
                .and_then(|values| values.get(index as usize))
                .ok_or(RegisterLinkError::Overflow("import index"))?,
        };
        if resolved.target.kind() != kind {
            return Err(RegisterLinkError::ImportKind {
                owner: object.to_string(),
                name: format!("reference {reference:?}"),
                expected: kind,
                actual: resolved.target.kind(),
            });
        }
        Ok(resolved)
    }

    pub(super) fn canonical_target_name(
        &self,
        objects: &[&RelocatableObject],
        object: usize,
        target: DefinitionTarget,
        fallback: &str,
    ) -> String {
        objects[object]
            .definitions
            .iter()
            .find(|definition| definition.target == target)
            .map_or_else(
                || fallback.to_ascii_lowercase(),
                |definition| definition.name.clone(),
            )
    }

    #[allow(dead_code, reason = "kept for deterministic symbol diagnostics")]
    pub(super) fn definition_count(&self) -> usize {
        self.definitions.len()
    }
}

fn matching_layout_definition(
    objects: &[&RelocatableObject],
    left: (usize, usize),
    right: (usize, usize),
) -> bool {
    let left_definition = &objects[left.0].definitions[left.1];
    let right_definition = &objects[right.0].definitions[right.1];
    match (left_definition.target, right_definition.target) {
        (DefinitionTarget::Record(left_index), DefinitionTarget::Record(right_index)) => {
            let Some(left_layout) = objects[left.0].records.get(left_index as usize) else {
                return false;
            };
            let Some(right_layout) = objects[right.0].records.get(right_index as usize) else {
                return false;
            };
            left_layout.fields.len() == right_layout.fields.len()
                && left_layout
                    .fields
                    .iter()
                    .zip(&right_layout.fields)
                    .all(|(left, right)| left.eq_ignore_ascii_case(right))
        }
        (DefinitionTarget::Enum(left_index), DefinitionTarget::Enum(right_index)) => {
            let Some(left_layout) = objects[left.0].enums.get(left_index as usize) else {
                return false;
            };
            let Some(right_layout) = objects[right.0].enums.get(right_index as usize) else {
                return false;
            };
            left_layout.variants.len() == right_layout.variants.len()
                && left_layout
                    .variants
                    .iter()
                    .zip(&right_layout.variants)
                    .all(|(left, right)| {
                        left.name.eq_ignore_ascii_case(&right.name)
                            && left.fields.len() == right.fields.len()
                            && left
                                .fields
                                .iter()
                                .zip(&right.fields)
                                .all(|(left, right)| left.eq_ignore_ascii_case(right))
                    })
        }
        _ => false,
    }
}

fn validate_shape(
    object: &RelocatableObject,
    definition: &ObjectDefinition,
    expected: &ImportShape,
) -> Result<(), String> {
    match (definition.target, expected) {
        (
            DefinitionTarget::Function(index),
            ImportShape::Function {
                arity,
                capture_count,
                returns_value,
            },
        ) => {
            let function = object
                .functions
                .get(index as usize)
                .ok_or_else(|| "definition has no callable implementation".to_string())?;
            let actual_returns = matches!(function.returns, fpas_unit::object::ObjectReturn::Value);
            if function.arity == *arity
                && function.capture_count == *capture_count
                && actual_returns == *returns_value
            {
                Ok(())
            } else {
                Err(format!(
                    "callable ABI is ({}, {}, value={actual_returns}), expected ({arity}, {capture_count}, value={returns_value})",
                    function.arity, function.capture_count
                ))
            }
        }
        (DefinitionTarget::Global(index), ImportShape::Global { mutable }) => {
            let global = object
                .globals
                .get(index as usize)
                .ok_or_else(|| "definition has no global slot".to_string())?;
            if global.mutable == *mutable {
                Ok(())
            } else {
                Err(format!(
                    "global mutability is {}, expected {mutable}",
                    global.mutable
                ))
            }
        }
        (DefinitionTarget::Record(index), ImportShape::Record { fields }) => {
            let record = object
                .records
                .get(index as usize)
                .ok_or_else(|| "definition has no record layout".to_string())?;
            let actual = record
                .fields
                .iter()
                .map(|field| field.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if actual == *fields {
                Ok(())
            } else {
                Err(format!(
                    "record fields are {:?}, expected {fields:?}",
                    record.fields
                ))
            }
        }
        (DefinitionTarget::Enum(index), ImportShape::Enum { variants }) => {
            let enumeration = object
                .enums
                .get(index as usize)
                .ok_or_else(|| "definition has no enum layout".to_string())?;
            let actual = enumeration
                .variants
                .iter()
                .map(|variant| {
                    (
                        variant.name.to_ascii_lowercase(),
                        variant
                            .fields
                            .iter()
                            .map(|field| field.to_ascii_lowercase())
                            .collect(),
                    )
                })
                .collect::<Vec<_>>();
            if actual == *variants {
                Ok(())
            } else {
                Err(format!(
                    "enum variants are {actual:?}, expected {variants:?}"
                ))
            }
        }
        _ => Err("symbol kind does not match import".to_string()),
    }
}
