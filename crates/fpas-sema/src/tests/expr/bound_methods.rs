//! Bound record method semantic tests.
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use super::super::{check_errors, check_ok};
use crate::analyze_with_types;

#[test]
fn bound_instance_function_is_callable_type() {
    check_ok(
        r#"program T;

type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

begin
  var C: Counter := Counter(Base := 10);
  var AddTen: function(Value: integer): integer := C.Add;
end program;
"#,
    );
}

#[test]
fn bound_instance_procedure_is_callable_type() {
    check_ok(
        r#"program T;

type Counter = record
  Base: integer;

  procedure Bump(Self: Counter);
  begin
    null;
  end procedure;
end record;

begin
  var C: Counter := Counter(Base := 1);
  var Op: procedure() := C.Bump;
end program;
"#,
    );
}

#[test]
fn rejects_binding_static_function_from_value() {
    let errors = check_errors(
        r#"program T;

type Point = record
  X: integer;

  static function Origin(): Point;
  begin
    return Point(X := 0);
  end function;
end record;

begin
  var P: Point := Point.Origin();
  var F: function(): Point := P.Origin;
end program;
"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("static function") && e.message.contains("bound")),
        "{errors:#?}"
    );
}

#[test]
fn rejects_binding_mutable_self_method() {
    let errors = check_errors(
        r#"program T;

type Counter = record
  Base: integer;

  procedure Inc(mutable Self: Counter);
  begin
    Self.Base := Self.Base + 1;
  end procedure;
end record;

begin
  var C: Counter := Counter(Base := 0);
  var Op: procedure() := C.Inc;
end program;
"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("mutable") && e.message.contains("bind")),
        "{errors:#?}"
    );
}

#[test]
fn bound_method_records_metadata() {
    let (program, parse_errors) = fpas_parser::parse(
        r#"program T;

type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

begin
  var C: Counter := Counter(Base := 10);
  var AddTen: function(Value: integer): integer := C.Add;
end program;
"#,
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert!(
        metadata
            .bound_methods
            .values()
            .any(|info| info.qualified_name.eq_ignore_ascii_case("Counter.Add")),
        "{:#?}",
        metadata.bound_methods
    );
}
