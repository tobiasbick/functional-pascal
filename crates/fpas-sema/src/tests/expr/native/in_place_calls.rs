//! Exercise named/positional InPlace checking with the value operation's independent fixtures.
use super::check_ok;

/// Reuses resolved value fixtures without duplicating the public parameter inventory.
pub(super) fn check_counterparts(cases: &[(String, String)]) {
    let resolved = cases
        .iter()
        .map(|(source, _)| {
            let (program, errors) = fpas_parser::parse(source);
            assert!(errors.is_empty());
            crate::analyze_with_types(&program)
                .fluent_calls
                .into_values()
                .map(|call| call.name)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for entry in crate::native_operations() {
        let crate::NativeLowering::InPlace(base) = entry.lowering else {
            continue;
        };
        let Some(operation) = crate::native_operation_by_implementation(base) else {
            // Capacity-only mutations have no public value counterpart; byte-buffer
            // contract tests exercise their explicit mutation API independently.
            assert!(matches!(
                entry.name,
                "ReserveInPlace" | "TrimCapacityInPlace"
            ));
            continue;
        };
        let index = resolved
            .iter()
            .position(|targets| targets.iter().any(|name| name == base))
            .expect("value fixture");
        for source in [cases[index].0.as_str(), cases[index].1.as_str()] {
            let updated = if matches!(
                (entry.signature)().return_type.as_ref(),
                crate::Ty::Result(..)
            ) {
                let original = operation
                    .signature_for(None)
                    .return_type
                    .to_string()
                    .replace("Std.Bytes.", "Bytes.");
                let result = (entry.signature)()
                    .return_type
                    .to_string()
                    .replace("Std.Bytes.", "Bytes.");
                source.replace(
                    &format!("const Output: {original}"),
                    &format!("const Output: {result}"),
                )
            } else if entry.name == "PopInPlace" {
                source.replace("ArrayPopResult of integer", "integer")
            } else {
                let (before, result) = source.split_once("const Output:").expect("result binding");
                let (_, call) = result.split_once(" := ").expect("result initializer");
                format!("{before}{call}")
            };
            let updated = updated.replace(
                &format!("Receiver.{}(", operation.name),
                &format!("Receiver.{}(", entry.name),
            );
            let updated = if base == "Std.Dictionaries.Map" {
                // The independent value fixture changes type; the mutating form must retain it.
                updated.replace(
                    "function(integer: integer): string begin return 'a'; end function",
                    "function(V: integer): integer begin return V; end function",
                )
            } else {
                updated
            };
            check_ok(&updated);
        }
    }
}
