//! Shared forward graphs, terminating alternatives, and mandatory cycle diagnostics.
//! See `docs/pascal/language/types/declaration-order.md`.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_NO_FINITE_TYPE_VALUE;

#[test]
fn legal_shared_record_dag_is_accepted_without_constructing_values() {
    let depth = 40;
    let mut source = "program FiniteDag;".to_string();
    for i in 0..depth {
        source.push_str(&format!(
            "type R{i} = record Left: R{}; Right: R{}; end record;",
            i + 1,
            i + 1
        ));
    }
    source.push_str(&format!(
        "type R{depth} = record Value: integer; end record; begin end."
    ));
    check_ok(&source);
}

#[test]
fn finite_mutual_alternatives_are_independent_of_root_and_variant_order() {
    for choice in [
        "Recursive(Next: Node); Empty;",
        "Empty; Recursive(Next: Node);",
    ] {
        let declarations = [
            "type Node = record Left: Choice; Right: Choice; end record;".to_string(),
            format!("type Choice = enum {choice} end enum;"),
            "type Alias = Node;".to_string(),
        ];
        for order in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            check_ok(&format!(
                "program T; {} {} {} begin end.",
                declarations[order[0]], declarations[order[1]], declarations[order[2]]
            ));
        }
    }
}

#[test]
fn mandatory_failure_skips_shared_finite_recursive_alternatives_in_its_witness() {
    let errors = check_errors(
        "program T;
        type Root = record Left: Good; Right: Good; Required: Bad; end record;
        type Good = enum Recursive(Next: Root); Stop(Value: Result of (Good, string)); end enum;
        type Bad = record Next: Bad; end record;
        begin end.",
    );
    let error = errors
        .iter()
        .find(|error| {
            error.code == SEMA_NO_FINITE_TYPE_VALUE && error.message.starts_with("Type `Root`")
        })
        .expect("root diagnostic");
    assert!(
        error.message.contains("Root.Required -> Bad.Next -> Bad"),
        "{error:?}"
    );
    assert!(
        !error.message.contains("Root.Left") && !error.message.contains("Root.Right"),
        "{error:?}"
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.code == SEMA_NO_FINITE_TYPE_VALUE
                && error.message.starts_with("Type `Good`")),
        "{errors:?}"
    );
}

#[test]
fn mandatory_result_and_shared_enum_cycles_report_each_declared_type() {
    for definitions in [
        "type A = record Value: Result of (B, B); end record;
        type B = enum Left(Next: A); Right(Next: A); end enum;",
        "type A = enum Left(Next: B); Right(Next: B); end enum;
        type B = record Left: A; Right: A; end record;",
    ] {
        let errors = check_errors(&format!("program T; {definitions} begin end."));
        let cycles: Vec<_> = errors
            .iter()
            .filter(|error| error.code == SEMA_NO_FINITE_TYPE_VALUE)
            .collect();
        assert_eq!(cycles.len(), 2, "{errors:?}");
        for cycle in cycles {
            assert!(
                cycle.message.contains("A.") && cycle.message.contains("B."),
                "{cycle:?}"
            );
            assert!(
                cycle
                    .help
                    .as_ref()
                    .is_some_and(|help| help.contains("Option")),
                "{cycle:?}"
            );
        }
    }
}
