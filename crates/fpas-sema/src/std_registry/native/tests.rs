//! Catalog consistency and migration-coverage checks independent of lexical imports.

use super::*;
use std::collections::HashSet;

mod handbook;

#[test]
fn catalog_accounts_for_all_existing_distinct_operations_and_three_empty_checks() {
    let entries = native_operations().copied().collect::<Vec<_>>();
    validate_native_catalog(&entries).expect("valid catalog");
    assert_eq!(entries.len(), 78);
    let old_names = [
        "Std.Str",
        "Std.Arrays",
        "Std.Dictionaries",
        "Std.Options",
        "Std.Results",
    ]
    .into_iter()
    .flat_map(fpas_std::std_unit_symbols)
    .collect::<HashSet<_>>();
    assert_eq!(old_names.len(), 75);
    for old in old_names {
        assert!(
            entries.iter().any(|entry| entry.implementation == *old),
            "unaccounted operation {old}"
        );
    }
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.lowering == NativeLowering::IsEmpty)
            .count(),
        3
    );
    assert!(entries.iter().all(|entry| entry.documentation.len() > 10));
}

#[test]
fn duplicates_are_rejected_case_insensitively_including_specialized_arrays() {
    let original =
        *native_operation(&Ty::Array(Box::new(Ty::Integer)), "Length").expect("array length");
    let duplicate = NativeOperation {
        name: "lEnGtH",
        receiver: NativeReceiver::StringArray,
        implementation: "Private.Duplicate",
        ..original
    };
    assert!(validate_native_catalog(&[original, duplicate]).is_err());
}

#[test]
fn extensions_cannot_change_shared_argument_roles_or_restore_synonymous_names() {
    let string = *native_operation(&Ty::String, "Slice").expect("string slice");
    let array = *native_operation(&Ty::Array(Box::new(Ty::Integer)), "Slice").expect("array slice");
    let reversed = NativeOperation {
        signature: || {
            function(
                vec![p("Len", Ty::Integer), p("Start", Ty::Integer)],
                Ty::String,
                false,
            )
        },
        ..string
    };
    assert!(validate_native_catalog(&[array, reversed]).is_err());
    for name in ["sIzE", "Count", "Substring"] {
        let synonym = NativeOperation { name, ..string };
        assert!(validate_native_catalog(&[synonym]).is_err());
    }
}

#[test]
fn canonical_collection_names_parameter_roles_and_mutation_modes_are_stable() {
    let types = [
        Ty::String,
        Ty::Array(Box::new(Ty::Integer)),
        Ty::Dict(Box::new(Ty::String), Box::new(Ty::Integer)),
    ];
    for ty in &types {
        for (name, result) in [("Length", Ty::Integer), ("IsEmpty", Ty::Boolean)] {
            let operation = native_operation(ty, name).expect("canonical operation");
            let signature = operation.signature_for(Some(ty));
            assert!(signature.params.is_empty());
            assert_eq!(*signature.return_type, result);
        }
        for old in ["Size", "Count", "Substring"] {
            assert!(native_operation(ty, old).is_none());
        }
        let reduce = native_operation(ty, "Reduce").expect("reduce");
        assert_eq!(
            (reduce.signature)()
                .params
                .iter()
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>(),
            ["Init", "F"]
        );
    }
    for ty in &types[..2] {
        assert_eq!(
            (native_operation(ty, "Slice").expect("slice").signature)()
                .params
                .iter()
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>(),
            ["Start", "Len"]
        );
    }
    for entry in native_operations() {
        assert!(
            (entry.signature)()
                .params
                .iter()
                .all(|param| !param.is_var())
        );
        assert_eq!((entry.signature)().variadic, entry.name == "Format");
        assert_eq!(
            entry.receiver_mode() == ParamMode::Var,
            matches!(entry.name, "Push" | "Pop")
        );
    }
}
