use super::rejects;
use crate::tests::check_ok;

#[test]
fn generic_default_calls_enforce_purity_only_when_selected() {
    let declarations = "
        pure function Empty of (T)(): array of (T); begin return []; end function;
        type Box of (T) = record Factory: function(): array of (T) := Empty; end record;
        type Outer of (T) = record Inner: Box of (T) := Box(); end record;";
    check_ok(&format!("program Main; {declarations}
        begin const Data: Outer of (integer) := Outer();
          const Resource: Box of (channel of (integer)) := Box(Factory := function(): array of (channel of (integer)) begin return []; end function);
          discard Data; discard Resource;
        end program;"));
    for declaration in ["Box", "Outer"] {
        rejects(&format!("program Main; {declarations}
            begin const Value: {declaration} of (channel of (integer)) := {declaration}(); discard Value; end program;"));
    }
    rejects(
        "program Main;
        function Make(): Box of (channel of (integer)); begin return Box(); end function;
        pure function Empty of (T)(): array of (T); begin return []; end function;
        type Box of (T) = record Factory: function(): array of (T) := Empty; end record;
        begin discard Make(); end program;",
    );
}

#[test]
fn defaults_allow_absent_handlers_and_empty_resource_collections() {
    check_ok(
        "program Main;
        type Handler = procedure();
        type Options of (T) = record
          OnClick: Option of (Handler) := Option.None;
          Items: array of (T) := [];
          Resources: array of (channel of (integer)) := [];
          Action: Handler := procedure() begin null; end procedure;
        end record;
        begin const Value: Options of (Handler) := Options(); discard Value; end program;",
    );
}

#[test]
fn defaults_reject_impure_evaluation_without_restricting_field_result_types() {
    for initializer in ["Ordinary()", "Mutable", "Action"] {
        let ty = if initializer == "Action" {
            "procedure()"
        } else {
            "integer"
        };
        rejects(&format!(
            "program Main;
            var Mutable: integer := 1;
            function Ordinary(): integer; begin return 1; end function;
            procedure Action(); begin null; end procedure;
            type Options = record Value: {ty} := {initializer}; end record;
            begin null; end program;"
        ));
    }
    check_ok(
        "program Main;
        pure function Initial(): integer; begin return 42; end function;
        type Options = record Value: integer := Initial(); end record;
        begin const Value := Options(); discard Value; end program;",
    );
}
