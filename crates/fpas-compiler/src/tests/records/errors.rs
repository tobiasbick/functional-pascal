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
