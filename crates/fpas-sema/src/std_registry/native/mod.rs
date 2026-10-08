//! Fixed operations of built-in types, shared by checking, lowering, and editors.
//!
//! **Documentation:** `docs/future/improve-syntax/ap06-dot-call-targets/catalog.md`

mod array;
mod dictionary;
mod factory;
mod option;
mod result;
mod string;
mod validation;

use crate::types::{FunctionTy, ParamMode, ParamTy, Ty};

/// Static receiver shape selecting a built-in operation without lexical lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeReceiver {
    /// Unicode scalar sequence.
    String,
    /// Array with any known element type.
    Array,
    /// Array restricted to string elements (`Join`).
    StringArray,
    /// Dictionary with known key and value types.
    Dictionary,
    /// Optional value.
    Option,
    /// Success or error value.
    Result,
    /// Factory written on the built-in type name `string`.
    StringFactory,
    /// Factory written on the `array` keyword.
    ArrayFactory,
}

/// Existing runtime operation or composition used to implement a catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLowering {
    /// Dispatch to the existing intrinsic identity.
    Intrinsic,
    /// Compare the existing length intrinsic result with zero.
    IsEmpty,
}

/// One case-insensitive public name and signature for a static receiver shape.
#[derive(Debug, Clone, Copy)]
pub struct NativeOperation {
    /// Receiver or factory owner.
    pub receiver: NativeReceiver,
    /// Canonical public operation name.
    pub name: &'static str,
    /// Private implementation identity; never registers a public free routine.
    pub implementation: &'static str,
    /// Explicit parameters and result; the instance receiver is excluded.
    pub signature: fn() -> FunctionTy,
    /// Runtime implementation strategy.
    pub lowering: NativeLowering,
    /// Public behavior summary used by editor tooling.
    pub documentation: &'static str,
}

impl NativeReceiver {
    /// Whether a concrete static receiver has this shape.
    #[must_use]
    pub fn accepts(self, ty: &Ty) -> bool {
        match (self, ty) {
            (Self::String, Ty::String)
            | (Self::Array, Ty::Array(_))
            | (Self::Dictionary, Ty::Dict(_, _))
            | (Self::Option, Ty::Option(_))
            | (Self::Result, Ty::Result(_, _)) => true,
            (Self::StringArray, Ty::Array(inner)) => **inner == Ty::String,
            _ => false,
        }
    }

    /// Public owner spelling used for signatures and documentation.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::String | Self::StringFactory => "string",
            Self::Array | Self::ArrayFactory => "array",
            Self::StringArray => "array of string",
            Self::Dictionary => "dict",
            Self::Option => "Option",
            Self::Result => "Result",
        }
    }
}

impl NativeOperation {
    /// Receiver mode required by this operation; explicit parameters remain read-only.
    #[must_use]
    pub fn receiver_mode(self) -> ParamMode {
        super::intrinsic_std_receiver_mode(self.implementation)
    }

    /// Returns an editor signature specialized with the receiver's known type arguments.
    #[must_use]
    pub fn signature_for(self, receiver: Option<&Ty>) -> FunctionTy {
        let mut bindings = std::collections::HashMap::new();
        match receiver {
            Some(Ty::Array(inner) | Ty::Option(inner)) => {
                bindings.insert("t".into(), (**inner).clone());
            }
            Some(Ty::Dict(key, value)) => {
                bindings.insert("k".into(), (**key).clone());
                bindings.insert("v".into(), (**value).clone());
            }
            Some(Ty::Result(ok, error)) => {
                bindings.insert("t".into(), (**ok).clone());
                bindings.insert("e".into(), (**error).clone());
            }
            _ => {}
        }
        let signature = Ty::Function((self.signature)());
        let Ty::Function(signature) =
            crate::check::Checker::substitute_type_params(&signature, &bindings)
        else {
            unreachable!()
        };
        signature
    }

    /// Whether operation-specific checking supplies generic callback constraints.
    pub(crate) fn polymorphic(self) -> bool {
        !self.implementation.starts_with("Std.Str.")
            || matches!(self.name, "Map" | "Filter" | "Reduce")
    }
}

/// Every native entry, including specialized receivers and the two factories.
pub fn native_operations() -> impl Iterator<Item = &'static NativeOperation> {
    [
        string::OPERATIONS,
        array::OPERATIONS,
        dictionary::OPERATIONS,
        option::OPERATIONS,
        result::OPERATIONS,
        factory::OPERATIONS,
    ]
    .into_iter()
    .flatten()
}

/// Resolves a native dot operation by static type and name alone.
#[must_use]
pub fn native_operation(ty: &Ty, name: &str) -> Option<&'static NativeOperation> {
    native_operations()
        .find(|entry| entry.receiver.accepts(ty) && entry.name.eq_ignore_ascii_case(name))
}

/// Resolves one of the two keyword-owned factories.
#[must_use]
pub fn native_factory(owner: &str, name: &str) -> Option<&'static NativeOperation> {
    factory::OPERATIONS.iter().find(|entry| {
        entry.receiver.label().eq_ignore_ascii_case(owner) && entry.name.eq_ignore_ascii_case(name)
    })
}

/// Looks up a private implementation identity for lowering and editor signatures.
#[must_use]
pub fn native_operation_by_implementation(name: &str) -> Option<&'static NativeOperation> {
    native_operations().find(|entry| entry.implementation.eq_ignore_ascii_case(name))
}

/// Gives the canonical replacement for a removed public routine or reference.
#[must_use]
pub fn native_migration_hint(name: &str) -> Option<String> {
    let entry = native_operations().find(|entry| {
        entry.implementation.eq_ignore_ascii_case(name)
            || !name.contains('.')
                && entry
                    .implementation
                    .rsplit('.')
                    .next()
                    .is_some_and(|short| short.eq_ignore_ascii_case(name))
    })?;
    let prefix = match entry.receiver {
        NativeReceiver::StringFactory | NativeReceiver::ArrayFactory => entry.receiver.label(),
        _ => "Value",
    };
    Some(format!(
        "Use `{prefix}.{}(…)`; built-in type operations are available without imports. The former five type-helper units and their free routine references have been removed.",
        entry.name
    ))
}

fn p(name: &str, ty: Ty) -> ParamTy {
    ParamTy::value(name, ty)
}
fn function(params: Vec<ParamTy>, return_type: Ty, variadic: bool) -> FunctionTy {
    FunctionTy {
        type_params: vec![],
        params,
        return_type: Box::new(return_type),
        variadic,
    }
}

/// Validates canonical names, corresponding argument roles, and unique identities.
///
/// Factories have their own owner namespace. `array of string` shares instance
/// operations with generic arrays, so specialization cannot introduce an overload.
pub fn validate_native_catalog(entries: &[NativeOperation]) -> Result<(), String> {
    for (index, entry) in entries.iter().enumerate() {
        validation::validate_entry(entry)?;
        for other in &entries[..index] {
            let overlap = entry.receiver == other.receiver
                || matches!(
                    (entry.receiver, other.receiver),
                    (NativeReceiver::Array, NativeReceiver::StringArray)
                        | (NativeReceiver::StringArray, NativeReceiver::Array)
                );
            if overlap && entry.name.eq_ignore_ascii_case(other.name) {
                return Err(format!(
                    "Duplicate native operation {}.{}",
                    entry.receiver.label(),
                    entry.name
                ));
            }
            if entry
                .implementation
                .eq_ignore_ascii_case(other.implementation)
            {
                return Err(format!(
                    "Duplicate native implementation {}",
                    entry.implementation
                ));
            }
            validation::validate_shared_roles(entry, other)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
