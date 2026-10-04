//! Pure signatures are validated after complete recursive nominal headers.

use super::rejects;
use crate::tests::check_ok;

#[test]
fn pure_signatures_accept_recursive_and_forward_resource_free_types() {
    check_ok(
        "program Main;
        type Node = record Callback: pure function(Value: Node): integer; end record;
        type ReadLater = pure function(Value: Later): integer;
        type Later = record Next: Option of (Later); Reader: ReadLater; end record;
        type Tree of (T) = record
          Value: T;
          Visit: pure function(Value: Tree of (T)): integer;
          Children: array of (Tree of (array of (T)));
        end record;
        begin null; end program;",
    );
}

#[test]
fn pure_signatures_reject_forbidden_components_behind_forward_references() {
    for component in ["procedure()", "channel of (integer)", "function(): integer"] {
        rejects(&format!(
            "program Main;
            type ReadLater = pure function(Value: Later): integer;
            type Later = record Value: {component}; end record;
            begin null; end program;"
        ));
    }
}

#[test]
fn growing_recursive_arguments_cannot_hide_a_forbidden_pure_signature() {
    rejects(
        "program Main;
        type Node of (T) = record
          Reader: pure function(Value: T): integer;
          Next: Option of (Node of (channel of (T)));
        end record;
        begin null; end program;",
    );
}
