//! Portable native-error schemas and internal generic construction requirements.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

mod budget;
mod requests;
mod substitution;
mod validation;

pub use budget::ErrorInstantiationBudget;
pub(crate) use substitution::substitute_identity;

use crate::interface::TypeConstraint;
pub use fpas_bytecode::{ErrorInstanceKey, IdentityParameter, NominalKind, TypeIdentity};

/// Complete error declaration retained for source-less instantiation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorSchema {
    /// Canonical defining declaration identity.
    pub declaration: String,
    /// Defining spelling used by automatic output.
    pub display_name: String,
    /// Declared and enclosing generic parameter bounds in substitution order.
    pub parameters: Vec<Option<TypeConstraint>>,
    /// Nonempty declaration-ordered variants.
    pub variants: Vec<ErrorSchemaVariant>,
}

/// One error variant before concrete type substitution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorSchemaVariant {
    /// Source variant spelling.
    pub name: String,
    /// Fields in declaration order.
    pub fields: Vec<ErrorSchemaField>,
    /// Optional nonempty literal label.
    pub message: Option<String>,
    /// Designated Cause slot, absent for a leaf.
    pub cause: Option<u16>,
}

/// A payload field with its essential semantic type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorSchemaField {
    /// Original field spelling.
    pub name: String,
    /// Alias-free field type, possibly referring to schema parameters.
    pub ty: TypeIdentity,
}

/// A constructor function requested by checked concrete or generic code.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ErrorConstructorRequest {
    /// Fully applied family, symbolic only in a generic requirement.
    pub instance: ErrorInstanceKey,
    /// Declaration-ordered variant index.
    pub variant: u16,
}

/// Hidden readonly function parameters required by one erased generic body.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorConstructorSignature {
    /// Number of enclosing semantic type parameters available for substitution.
    pub type_parameters: u16,
    /// Constructor parameters in stable ABI order, following explicit arguments.
    pub requirements: Vec<ErrorConstructorRequest>,
}

/// Construction ABI of an exported generic, separate from its source signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorRoutineConstructors {
    /// Canonical routine identity.
    pub routine: String,
    /// Hidden constructor requirements, including transitive private callees.
    pub signature: ErrorConstructorSignature,
}

/// Essential public error declarations and generic construction ABI.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorInterface {
    /// Public and hidden constructor-required declarations, sorted by canonical identity.
    pub schemas: Vec<ErrorSchema>,
    /// Public routine requirements, sorted by routine identity.
    pub routines: Vec<ErrorRoutineConstructors>,
    /// Private and public nominal fields needed by hidden constructor arguments.
    pub equality_declarations: Vec<crate::equality::EqualityDeclaration>,
}

impl ErrorInterface {
    /// Whether this interface retains the ordinary pre-error shape.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.schemas.is_empty() && self.routines.is_empty() && self.equality_declarations.is_empty()
    }

    /// Validate error schemas and construction ABI before hashing or installation.
    ///
    /// # Errors
    /// Returns an invalid declaration, generic bound or requirement invariant.
    pub fn validate(&self) -> Result<(), &'static str> {
        crate::equality::EqualityDeclarations::new(self.equality_declarations.iter().cloned())?;
        for schema in &self.schemas {
            schema.validate()?;
        }
        for routine in &self.routines {
            routine.signature.validate()?;
        }
        if self
            .equality_declarations
            .windows(2)
            .any(|pair| pair[0].identity >= pair[1].identity)
            || self
                .schemas
                .windows(2)
                .any(|pair| pair[0].declaration >= pair[1].declaration)
            || self
                .routines
                .windows(2)
                .any(|pair| pair[0].routine >= pair[1].routine)
        {
            return Err("error interface identities must be unique and sorted");
        }
        Ok(())
    }
}

impl ErrorConstructorSignature {
    /// Validate bounded symbolic construction requirements.
    ///
    /// # Errors
    /// Rejects unknown parameters, repeated requirements and oversized call ABIs.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.requirements.len() > fpas_bytecode::limits::MAX_CALL_ARGUMENTS {
            return Err("too many hidden error constructors");
        }
        let mut seen = std::collections::BTreeSet::new();
        for request in &self.requirements {
            request.instance.validate(self.type_parameters)?;
            if !seen.insert(request) {
                return Err("duplicate hidden error constructor");
            }
        }
        Ok(())
    }
}

/// Stable internal symbol for one concrete instance, with full key equality at linking.
///
/// # Errors
/// Rejects nonconcrete or unbounded keys and serialization failure.
pub fn instance_symbol(key: &ErrorInstanceKey) -> Result<String, String> {
    key.validate(0).map_err(str::to_owned)?;
    let bytes = serde_json::to_vec(key).map_err(|error| error.to_string())?;
    Ok(format!(
        "{}.$error.{:?}",
        key.declaration,
        crate::Digest::of(bytes)
    ))
}

/// Stable internal function symbol; never substitutes a digest for full key equality.
///
/// # Errors
/// Rejects an invalid concrete constructor request.
pub fn constructor_symbol(request: &ErrorConstructorRequest) -> Result<String, String> {
    Ok(format!(
        "{}.$constructor.{}",
        instance_symbol(&request.instance)?,
        request.variant
    ))
}
