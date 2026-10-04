//! Generic declarations and identity-based type argument substitution.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use std::collections::HashMap;

use super::{Ty, TypeConstraint};

/// Persistent identity of a declared generic parameter.
pub use fpas_unit::interface::GenericParameterId;

/// A resolved generic type parameter with its declaration identity and constraint.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParamDef {
    /// Source spelling used in diagnostics.
    pub name: String,
    /// Optional capability required of caller-supplied types.
    pub constraint: Option<TypeConstraint>,
    /// Declaring source parameter, distinct from similarly named parameters.
    pub identity: GenericParameterId,
}

/// Concrete or enclosing-generic arguments indexed by their declared parameters.
pub(crate) type TypeArguments = HashMap<GenericParameterId, Ty>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::types::{FunctionTy, ParamTy};

    fn parameter(name: &str, offset: u64) -> GenericParamDef {
        GenericParamDef {
            name: name.to_owned(),
            constraint: None,
            identity: GenericParameterId {
                unit: Some("demo.model".to_owned()),
                source_id: 0,
                offset,
            },
        }
    }

    #[test]
    fn generic_parameter_identity_controls_compatibility() {
        let outer = Ty::GenericParam(Arc::new(parameter("T", 10)));
        let inner = Ty::GenericParam(Arc::new(parameter("T", 20)));
        let same = Ty::GenericParam(Arc::new(parameter("t", 10)));
        assert!(!outer.compatible_with(&inner));
        assert!(!outer.assignment_compatible_with(&inner));
        assert!(outer.compatible_with(&same));
        assert!(outer.assignment_compatible_with(&same));
    }

    #[test]
    fn generic_parameter_identity_substitutes_only_the_declared_parameter() {
        let outer = Ty::GenericParam(Arc::new(parameter("T", 10)));
        let inner = parameter("T", 20);
        let signature = Ty::Function(FunctionTy {
            pure: false,
            type_params: vec![inner.clone()],
            params: vec![ParamTy {
                mutable: false,
                name: "Value".to_owned(),
                ty: Ty::Array(Box::new(Ty::GenericParam(Arc::new(inner.clone())))),
            }],
            return_type: Box::new(Ty::Option(Box::new(outer.clone()))),
            variadic: false,
        });
        let substituted =
            signature.substitute(&TypeArguments::from([(inner.identity, Ty::Integer)]));
        let Ty::Function(signature) = substituted else {
            panic!("expected function signature");
        };
        assert_eq!(signature.params[0].ty, Ty::Array(Box::new(Ty::Integer)));
        assert_eq!(*signature.return_type, Ty::Option(Box::new(outer)));
    }
}
