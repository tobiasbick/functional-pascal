//! Bound record method semantic tests.
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use super::super::{check_errors, check_ok};
use crate::analyze_with_types;

#[test]
fn bound_instance_function_is_callable_type() {
    check_ok(
        "\
program T;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
begin
  var C: Counter := record Base := 10; end;
  var AddTen: function(Value: integer): integer := C.Add
end.",
    );
}

#[test]
fn bound_instance_procedure_is_callable_type() {
    check_ok(
        "\
program T;
type
  Counter = record
    Base: integer;
    procedure Bump(Self: Counter);
    begin
    end;
  end;
begin
  var C: Counter := record Base := 1; end;
  var Op: procedure() := C.Bump
end.",
    );
}

#[test]
fn rejects_binding_static_function_from_value() {
    let errors = check_errors(
        "\
program T;
type
  Point = record
    X: integer;
    static function Origin(): Point;
    begin
      return record X := 0; end
    end;
  end;
begin
  var P: Point := Point.Origin();
  var F: function(): Point := P.Origin
end.",
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
        "\
program T;
type
  Counter = record
    Base: integer;
    procedure Inc(mutable Self: Counter);
    begin
      Self.Base := Self.Base + 1
    end;
  end;
begin
  var C: Counter := record Base := 0; end;
  var Op: procedure() := C.Inc
end.",
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
        "\
program T;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
begin
  var C: Counter := record Base := 10; end;
  var AddTen: function(Value: integer): integer := C.Add
end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let (errors, _, _, _, _, _, _, bound) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(
        bound
            .values()
            .any(|info| info.qualified_name.eq_ignore_ascii_case("Counter.Add")),
        "{bound:#?}"
    );
}
