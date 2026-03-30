/// Negative tests for record types: field access errors, immutability, type mismatches.
///
/// **Documentation:** [docs/pascal/05-types.md](docs/pascal/05-types.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// IMMUTABILITY
// ═══════════════════════════════════════════════════════════════

#[test]
fn immutable_record_field_set_error() {
    let err = compile_err(
        "\
program ImmutRec;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  P.X := 99
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT);
}

// ═══════════════════════════════════════════════════════════════
// UNKNOWN FIELD
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_access_nonexistent_field() {
    let err = compile_err(
        "\
program BadField;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  Std.Console.WriteLn(P.Z)
end.",
    );
    assert!(
        err.message.to_lowercase().contains("field")
            || err.message.to_lowercase().contains("member")
            || err.message.contains("Z"),
        "expected unknown field error, got: {}",
        err.message
    );
}

// ═══════════════════════════════════════════════════════════════
// TYPE MISMATCH IN FIELD INITIALIZATION
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_field_type_mismatch() {
    let err = compile_err(
        "\
program FieldMismatch;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 'hello'; Y := 2; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

// ═══════════════════════════════════════════════════════════════
// UNKNOWN TYPE IN RECORD
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_unknown_field_type() {
    let err = compile_err(
        "\
program UnknownFieldType;
type Bad = record X: Nonexistent; end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

// ═══════════════════════════════════════════════════════════════
// METHOD ERRORS
// ═══════════════════════════════════════════════════════════════

#[test]
fn method_wrong_arg_count() {
    let err = compile_err(
        "\
program MethodArgCount;
type Num = record
  V: integer;
  function Add(Self: Num; Other: Num): integer;
  begin
    return Self.V + Other.V
  end;
end;
begin
  var N: Num := record V := 1; end;
  Std.Console.WriteLn(N.Add())
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

#[test]
fn record_literal_missing_field_reports_error() {
    let err = compile_err(
        "\
program MissingField;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD);
}

#[test]
fn record_literal_extra_field_reports_error() {
    let err = compile_err(
        "\
program ExtraField;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; Z := 3; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}

#[test]
fn record_literal_duplicate_field_reports_error() {
    let err = compile_err(
        "\
program DuplicateField;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; X := 2; end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION
    );
}

#[test]
fn record_method_without_self_parameter_is_rejected() {
    let err = compile_err(
        "\
program MissingSelf;
type Point = record
  X: integer;
  function Sum(): integer;
  begin
    return X
  end;
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Self: Point"),
        "expected explicit Self requirement, got: {}",
        err.message
    );
}

#[test]
fn record_method_with_wrong_first_parameter_name_is_rejected() {
    let err = compile_err(
        "\
program WrongSelfName;
type Point = record
  X: integer;
  function Sum(This: Point): integer;
  begin
    return This.X
  end;
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Self: Point"),
        "expected explicit Self requirement, got: {}",
        err.message
    );
}

#[test]
fn record_method_with_wrong_self_type_is_rejected() {
    let err = compile_err(
        "\
program WrongSelfType;
type Other = record X: integer; end;
type Point = record
  X: integer;
  function Sum(Self: Other): integer;
  begin
    return Self.X
  end;
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Self: Point"),
        "expected explicit Self requirement, got: {}",
        err.message
    );
}
