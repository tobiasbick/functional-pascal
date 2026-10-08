//! Canonical collection vocabulary and corresponding explicit parameter roles.

use super::{NativeOperation, NativeReceiver};
use crate::types::Ty;

/// Checks canonical collection count and slice names and result shapes.
pub(super) fn validate_entry(entry: &NativeOperation) -> Result<(), String> {
    let collection = matches!(
        entry.receiver,
        NativeReceiver::String
            | NativeReceiver::Array
            | NativeReceiver::StringArray
            | NativeReceiver::Dictionary
    );
    if collection
        && ["Size", "Count", "Substring"]
            .iter()
            .any(|name| entry.name.eq_ignore_ascii_case(name))
    {
        return Err(format!(
            "Noncanonical native operation {}.{}; use Length or Slice",
            entry.receiver.label(),
            entry.name
        ));
    }
    if collection {
        let result = if entry.name.eq_ignore_ascii_case("Length") {
            Some(Ty::Integer)
        } else if entry.name.eq_ignore_ascii_case("IsEmpty") {
            Some(Ty::Boolean)
        } else {
            None
        };
        if let Some(result) = result {
            let signature = (entry.signature)();
            if !signature.params.is_empty()
                || signature.variadic
                || *signature.return_type != result
            {
                return Err(format!("Invalid collection signature for {}", entry.name));
            }
        }
    }
    Ok(())
}

/// Checks matching explicit roles while allowing substring versus element inputs.
pub(super) fn validate_shared_roles(
    entry: &NativeOperation,
    other: &NativeOperation,
) -> Result<(), String> {
    if !entry.name.eq_ignore_ascii_case(other.name) {
        return Ok(());
    }
    let first = (entry.signature)();
    let second = (other.signature)();
    let names = |signature: &crate::types::FunctionTy| {
        signature
            .params
            .iter()
            .map(|param| param.name.to_ascii_lowercase())
            .collect::<Vec<_>>()
    };
    let first_names = names(&first);
    let second_names = names(&second);
    // Substring membership/search takes Sub, while array membership/search
    // takes Value. Both have one input; the type-required distinction is public.
    let membership = ["Contains", "IndexOf"]
        .iter()
        .any(|name| entry.name.eq_ignore_ascii_case(name));
    let string_array = matches!(
        (entry.receiver, other.receiver),
        (NativeReceiver::String, NativeReceiver::Array)
            | (NativeReceiver::Array, NativeReceiver::String)
    );
    let different_membership = membership
        && string_array
        && ((first_names == ["sub"] && second_names == ["value"])
            || (first_names == ["value"] && second_names == ["sub"]));
    if (first_names != second_names && !different_membership) || first.variadic != second.variadic {
        return Err(format!(
            "Inconsistent explicit argument roles for {}.{} and {}.{}",
            entry.receiver.label(),
            entry.name,
            other.receiver.label(),
            other.name
        ));
    }
    Ok(())
}
