//! Essential nominal field declarations for source-less equality admission.

use fpas_code::{NominalKind, TypeIdentity};
use std::collections::{BTreeMap, BTreeSet};

/// A nominal declaration's fields, independent of optional debugger metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EqualityDeclaration {
    /// Alias-free defining identity and symbolic declaration arguments.
    pub identity: TypeIdentity,
    /// Number of declared parameters used by the field types.
    pub parameters: u16,
    /// Every record field or every possible enum payload field.
    #[serde(deserialize_with = "TypeIdentity::deserialize_list")]
    pub fields: Vec<TypeIdentity>,
}

impl EqualityDeclaration {
    /// Validate a bounded declaration before retaining it in an object.
    pub fn validate(&self) -> Result<(), &'static str> {
        let TypeIdentity::Nominal {
            kind, arguments, ..
        } = &self.identity
        else {
            return Err("equality declaration requires nominal identity");
        };
        if !matches!(
            kind,
            NominalKind::Record | NominalKind::Enum | NominalKind::OrdinalEnum | NominalKind::Error
        ) || arguments.len() != usize::from(self.parameters)
            || arguments
                .iter()
                .enumerate()
                .any(|(index, arg)| *arg != TypeIdentity::Parameter(index as u16))
        {
            return Err("equality declaration parameters disagree with identity");
        }
        if *kind == NominalKind::OrdinalEnum && !self.fields.is_empty() {
            return Err("ordinal equality declaration must not have payload fields");
        }
        TypeIdentity::validate_all(std::slice::from_ref(&self.identity), self.parameters)?;
        TypeIdentity::validate_all(&self.fields, self.parameters)
    }
}

/// Conflict-checked declarations shared by the compiler and source-less linker.
#[derive(Default)]
pub struct EqualityDeclarations(BTreeMap<String, EqualityDeclaration>);

impl EqualityDeclarations {
    /// Admit declarations, rejecting inconsistent copies of a nominal definition.
    pub fn new(
        declarations: impl IntoIterator<Item = EqualityDeclaration>,
    ) -> Result<Self, &'static str> {
        let mut result = Self::default();
        for schema in declarations {
            schema.validate()?;
            let TypeIdentity::Nominal { declaration, .. } = &schema.identity else {
                return Err("missing equality declaration");
            };
            if let Some(previous) = result.0.insert(declaration.clone(), schema.clone())
                && previous != schema
            {
                return Err("conflicting equality declarations");
            }
            if result.0.len() > fpas_code::limits::MAX_RECORD_LAYOUTS {
                return Err("too many equality declarations");
            }
        }
        Ok(result)
    }

    /// Return the deduplicated declarations in canonical defining-name order.
    pub fn into_declarations(self) -> Vec<EqualityDeclaration> {
        self.0.into_values().collect()
    }

    /// Determine equality from field declarations, with a bounded recursive graph.
    pub fn supports(&self, ty: &TypeIdentity) -> bool {
        if TypeIdentity::validate_all(std::slice::from_ref(ty), 0).is_err() {
            return false;
        }
        let mut pending = vec![ty.clone()];
        let mut visited = BTreeSet::new();
        let mut budget = fpas_code::limits::MAX_LAYOUT_FIELDS;
        let mut expansion = crate::errors::ErrorInstantiationBudget::default();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty.clone()) {
                continue;
            }
            if budget == 0 {
                return false;
            }
            budget -= 1;
            match &ty {
                TypeIdentity::Integer
                | TypeIdentity::Real
                | TypeIdentity::Boolean
                | TypeIdentity::String
                | TypeIdentity::Subrange { .. } => {}
                TypeIdentity::Option(inner) => pending.push(inner.as_ref().clone()),
                TypeIdentity::Result(ok, error) => {
                    pending.push(ok.as_ref().clone());
                    pending.push(error.as_ref().clone());
                }
                TypeIdentity::Nominal {
                    kind: NominalKind::Distinct | NominalKind::OrdinalEnum,
                    ..
                } => {}
                TypeIdentity::Nominal {
                    declaration,
                    arguments,
                    kind,
                } => {
                    let Some(schema) = self.0.get(declaration) else {
                        return false;
                    };
                    let TypeIdentity::Nominal {
                        kind: declared_kind,
                        ..
                    } = &schema.identity
                    else {
                        return false;
                    };
                    if kind != declared_kind || arguments.len() != usize::from(schema.parameters) {
                        return false;
                    }
                    if pending.len().saturating_add(schema.fields.len()) > budget {
                        return false;
                    }
                    for field in &schema.fields {
                        let Ok(field) =
                            crate::errors::substitute_identity(field, arguments, &mut expansion)
                        else {
                            return false;
                        };
                        pending.push(field);
                    }
                }
                _ => return false,
            }
        }
        true
    }
}
