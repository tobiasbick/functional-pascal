//! Enum identities, generic arguments, and associated-data variants.
//! See `docs/pascal/language/types/enums.md` and `docs/pascal/language/types/generics.md`.

use super::{FunctionTy, GenericParamDef, ParamTy, Ty, substitute_type_parameters};
use std::{collections::HashMap, sync::Arc};

/// Nominal enum declaration or application with resolved payload types.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumTy {
    /// Native error families retain concrete runtime identity and automatic output.
    pub is_error: bool,
    /// Case-preserving qualified enum name.
    pub name: String,
    /// Generic parameters in declaration order.
    pub type_params: Vec<GenericParamDef>,
    /// Actual arguments; empty for a declaration.
    pub type_args: Vec<Ty>,
    /// Declared variants in source order.
    pub variants: Vec<EnumVariantTy>,
}

/// A single enum variant with associated-data fields or an integer backing value.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantTy {
    /// Optional literal native-error message.
    pub message: Option<String>,
    /// Case-preserving variant name.
    pub name: String,
    /// Associated-data fields in declaration order.
    pub fields: Vec<(String, Ty)>,
    /// Declared or implicit integer value for a simple enum member.
    pub backing_value: Option<i64>,
}

impl EnumTy {
    /// Reserve an enum identity before resolving recursive payload references.
    pub fn header(name: String, type_params: Vec<GenericParamDef>) -> Self {
        Self {
            is_error: false,
            name,
            type_params,
            type_args: Vec::new(),
            variants: Vec::new(),
        }
    }

    /// Whether variants use tagged enum storage, including fieldless native errors.
    pub fn has_data(&self) -> bool {
        self.is_error
            || self
                .variants
                .iter()
                .any(|variant| !variant.fields.is_empty())
    }

    /// Return a payload constructor's readonly signature, retaining generic parameters.
    /// Fieldless variants are values rather than callable constructors.
    pub fn constructor_type(self: &Arc<Self>, variant_name: &str) -> Option<FunctionTy> {
        let variant = self
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))?;
        if variant.fields.is_empty() {
            return None;
        }
        Some(FunctionTy {
            type_params: if self.type_args.is_empty() {
                self.type_params.clone()
            } else {
                vec![]
            },
            params: variant
                .fields
                .iter()
                .map(|(name, ty)| ParamTy::value(name, ty.clone()))
                .collect(),
            return_type: Box::new(Ty::Enum(Arc::clone(self))),
            variadic: false,
        })
    }

    /// Substitute declaration parameters without expanding recursive identities.
    pub fn instantiate(&self, arguments: Vec<Ty>) -> Self {
        let bindings: HashMap<_, _> = self
            .type_params
            .iter()
            .zip(&arguments)
            .map(|(parameter, argument)| (parameter.name.to_ascii_lowercase(), argument.clone()))
            .collect();
        let mut instance = self.clone();
        instance.type_args = arguments;
        for variant in &mut instance.variants {
            for (_, field) in &mut variant.fields {
                *field = substitute_type_parameters(field, &bindings);
            }
        }
        instance
    }
}
