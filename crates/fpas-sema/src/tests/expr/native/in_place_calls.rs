//! Exercise named/positional InPlace checking with the value operation's independent fixtures.
use super::check_ok;

/// Reuses resolved value fixtures without duplicating the public parameter inventory.
pub(super) fn check_counterparts(cases: &[(&str, &str)]) {
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
        let index = resolved
            .iter()
            .position(|targets| targets.iter().any(|name| name == base))
            .expect("value fixture");
        let operation = crate::native_operation_by_implementation(base).expect("value operation");
        for source in [cases[index].0, cases[index].1] {
            let updated = if entry.name == "PopInPlace" {
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
