use super::{check_errors, check_ok};

#[test]
fn retired_words_can_name_values_functions_and_fields() {
    for name in [
        "static", "Property", "event", "Read", "Write", "Self", "Nil", "Assigned",
    ] {
        check_ok(&format!(
            "program P; type Box = record {name}: integer; end record; function {name}(Self: Box): integer; begin return Self.{name}; end function; begin const Value := {name}(Box({name} := 42)); end program;"
        ));
    }
}

#[test]
fn nil_and_assigned_have_no_implicit_language_binding() {
    for expression in ["nil", "Assigned(Value)"] {
        let errors = check_errors(&format!(
            "program P; const Value: Option of (integer) := Option.None; begin discard {expression}; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Undefined identifier")
                    || error.message.contains("Unknown function")),
            "{errors:#?}"
        );
    }
}
