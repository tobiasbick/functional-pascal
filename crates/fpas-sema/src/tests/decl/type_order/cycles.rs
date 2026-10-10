use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CYCLIC_TYPE_ALIAS, SEMA_NO_FINITE_TYPE_VALUE};

#[test]
fn alias_only_cycles_report_the_participating_names() {
    for definitions in [
        "type A = A;",
        "type A = B; type B = A;",
        "type A = array of B; type B = Option of A;",
        "type A = function(Value: Node): A; type Node = record Callback: A; end record;",
        "type A = Result of (Node, A); type Node = record Next: Option of A; end record;",
    ] {
        let errors = check_errors(&format!("program T; {definitions} begin end."));
        let error = errors
            .iter()
            .find(|error| error.code == SEMA_CYCLIC_TYPE_ALIAS)
            .unwrap();
        assert!(error.message.contains("A"));
        assert!(error.message.contains("->"));
    }
}

#[test]
fn mandatory_record_cycles_report_fields_and_types() {
    for definitions in [
        "type A = record Next: A; end record;",
        "type A = record Next: B; end record; type B = record Back: A; end record;",
        "type Alias = A; type A = record Next: Alias; end record;",
    ] {
        let errors = check_errors(&format!("program T; {definitions} begin end."));
        let error = errors
            .iter()
            .find(|error| error.code == SEMA_NO_FINITE_TYPE_VALUE)
            .unwrap();
        assert!(error.message.contains("A.Next"), "{error:?}");
        assert!(error.help.as_deref().unwrap().contains("Option"));
    }
}

#[test]
fn diagnostics_name_the_mandatory_cycle_after_a_finite_recursive_field() {
    let errors = check_errors(
        "program T;
      type Root = record First: Result of (Root, string); Required: Bad; end record;
      type Bad = record Next: Bad; end record;
      begin end.",
    );
    let error = errors
        .iter()
        .find(|error| {
            error.code == SEMA_NO_FINITE_TYPE_VALUE && error.message.starts_with("Type `Root`")
        })
        .unwrap();
    assert!(error.message.contains("Bad.Next"), "{error:?}");
    assert!(!error.message.contains("Root.First"), "{error:?}");
}

#[test]
fn enum_alternatives_must_have_a_finite_payload_path() {
    check_ok(
        "program T; type Chain = enum Empty; Link(Next: Chain); end enum;
      begin const Value: Chain := Chain.Link(Chain.Empty); discard Value; end.",
    );
    check_ok(
        "program T;
      type A = enum More(Next: B); end enum;
      type B = enum Empty; More(Next: A); end enum;
      begin const Value: A := A.More(B.Empty); discard Value; end.",
    );
    for definitions in [
        "type A = enum More(Next: A); end enum;",
        "type A = enum First(Next: A); Second(Next: B); end enum; type B = record Back: A; end record;",
        "type A = enum More(Next: B; Required: A); end enum; type B = enum Empty; end enum;",
    ] {
        let errors = check_errors(&format!("program T; {definitions} begin end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NO_FINITE_TYPE_VALUE)
        );
    }
}

#[test]
fn optional_containers_and_callable_signatures_end_layout_recursion() {
    check_ok(
        "program T; type Node = record Next: Option of Node;
      Children: array of Node; Values: dict of string to Node;
      Callback: function(Value: Node): Node;
      end record; begin end.",
    );
    check_ok(
        "program T; type Callback = function(Value: Node): Node;
      type Node = record Handler: Callback; end record; begin end.",
    );
}

#[test]
fn result_recursion_can_end_in_either_branch() {
    for payload in ["Result of (Node, string)", "Result of (string, Node)"] {
        check_ok(&format!(
            "program T; type Node = record Next: {payload}; end record; begin end."
        ));
    }
    let errors = check_errors(
        "program T; type Node = record Next: Result of (Node, Node); end record; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NO_FINITE_TYPE_VALUE)
    );
}
