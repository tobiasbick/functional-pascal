//! Postfix projections and calls preserve concrete result types and written arguments.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn fields_on_returned_records_and_aliases_keep_their_types() {
    check_ok(
        "program T;
      type Point = record X: integer; Y: integer; end record;
      type Alias = Point;
      function Make(): Alias; begin return Point(X := 4, Y := 5); end function;
      const X: integer := Make().X;
      const Y: integer := (Make()).Y;
      begin null; end program;",
    );
}

#[test]
fn indexing_returned_collections_preserves_element_types() {
    for (result, value, key, expected) in [
        ("array of (integer)", "[10, 20]", "1", "integer"),
        ("dict of (string, integer)", "['a': 1]", "'a'", "integer"),
        ("string", "'ab'", "0", "string"),
    ] {
        check_ok(&format!(
            "program T; function Make(): {result}; begin return {value}; end function;
          const Value: {expected} := Make()[{key}]; begin null; end program;"
        ));
    }
}

#[test]
fn missing_fields_and_calls_do_not_cascade_into_later_suffixes() {
    for suffix in [
        "Missing",
        "Missing.Another",
        "Missing()",
        "Missing().Another",
    ] {
        let errors = check_errors(&format!(
            "program T;
          type Point = record X: integer; end record;
          function Make(): Point; begin return Point(X := 1); end function;
          begin discard Make().{suffix}; end program;"
        ));
        assert_eq!(errors.len(), 1, "{suffix}: {errors:#?}");
        assert!(errors[0].message.contains("Missing"), "{errors:#?}");
    }
}

#[test]
fn scalar_fields_and_invalid_collection_indices_are_rejected() {
    for (result, value, suffix, message) in [
        ("integer", "1", ".X", "requires a record value"),
        (
            "array of (integer)",
            "[1]",
            "['x']",
            "Array index must be integer",
        ),
        ("integer", "1", "[0]", "not an array"),
    ] {
        let errors = check_errors(&format!(
            "program T;
          function Make(): {result}; begin return {value}; end function;
          begin discard Make(){suffix}; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.message.contains(message)),
            "{errors:#?}"
        );
    }
    let errors = check_errors(
        "program T; type Point = record X: integer; end record;
      function Make(): Point; begin return Point(X := 1); end function;
      begin discard Make()[0]; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("not an array")),
        "{errors:#?}"
    );
}

const CALLABLE_FIELDS: &str = "
type Point = record
  X: integer;
  Touch: procedure();
  Transform: function(Value: integer): integer;
end record;
function Make(): Point;
begin return Point(X := 1,
  Touch := procedure() begin null; end procedure,
  Transform := function(Value: integer): integer begin return Value + 1; end function);
end function;
";

#[test]
fn callable_fields_after_factories_take_only_explicit_arguments() {
    check_ok(&format!(
        "program T; {CALLABLE_FIELDS}
      begin Make().Touch(); const Value: integer := Make().Transform(2); end program;"
    ));
}

#[test]
fn procedure_fields_produce_no_value_and_must_finish_a_chain() {
    for body in [
        "const Value: integer := Make().Touch();",
        "Make().Touch().Transform(2);",
        "discard Make().Touch();",
    ] {
        let errors = check_errors(&format!(
            "program T; {CALLABLE_FIELDS} begin {body} end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("does not return a value")),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn postfix_statements_require_a_final_call_and_consumed_function_result() {
    for (body, message) in [
        ("Make().X;", "must end with a call"),
        ("Make().Transform(2);", "discard"),
    ] {
        let errors = check_errors(&format!(
            "program T; {CALLABLE_FIELDS} begin {body} end program;"
        ));
        assert!(
            errors.iter().any(|error| error.message.contains(message)
                || error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains(message))),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn generic_record_functions_and_factory_values_keep_concrete_results() {
    check_ok(
        "program T;
      type Value = record Number: integer; end record;
      type Box = record Number: integer; end record;
      function BoxMap of (T)(Receiver: Box; Transform: function(N: integer): T): T;
      begin return Transform(Receiver.Number); end function;
      function Identity of (T)(Input: T): T; begin return Input; end function;
      function Create(): Box; begin return Box(Number := 7); end function;
      function Wrap(N: integer): Value; begin return Value(Number := N); end function;
      function Number(N: integer): integer; begin return N; end function;
      const Scalar: integer := BoxMap(Create(), Number);
      const Field: integer := BoxMap(Create(), Wrap).Number;
      const Generic: integer := Identity(Wrap(9)).Number;
      begin
        const Factory: function(N: integer): Value := Wrap;
        const Indirect: integer := Factory(11).Number;
      end program;",
    );
}
