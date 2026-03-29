/// Tests for record default field values (`Field: Type := default_expr`).
///
/// **Documentation:** [docs/pascal/05-types.md](docs/pascal/05-types.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE: DEFAULT VALUES USED
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_default_used_when_field_omitted() {
    let out = compile_and_run(
        "\
program RecDefault;
type Point = record X: integer := 0; Y: integer := 99; end;
begin
  var P: Point := record X := 10; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "99"]);
}

#[test]
fn record_default_overridden_by_explicit_value() {
    let out = compile_and_run(
        "\
program RecOverride;
type Point = record X: integer := 0; Y: integer := 99; end;
begin
  var P: Point := record X := 5; Y := 7; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["5", "7"]);
}

#[test]
fn record_all_defaults_empty_literal() {
    let out = compile_and_run(
        "\
program RecAllDefaults;
type Config = record Enabled: boolean := false; Count: integer := 42; end;
begin
  var C: Config := record end;
  Std.Console.WriteLn(C.Enabled);
  Std.Console.WriteLn(C.Count)
end.",
    );
    assert_eq!(out.lines, vec!["false", "42"]);
}

#[test]
fn record_partial_defaults_multiple_types() {
    let out = compile_and_run(
        "\
program RecPartial;
type Settings = record Name: string := 'default'; Value: integer := 0; Active: boolean := false; end;
begin
  var S: Settings := record Name := 'custom'; end;
  Std.Console.WriteLn(S.Name);
  Std.Console.WriteLn(S.Value);
  Std.Console.WriteLn(S.Active)
end.",
    );
    assert_eq!(out.lines, vec!["custom", "0", "false"]);
}

#[test]
fn record_default_boolean_field() {
    let out = compile_and_run(
        "\
program RecBoolDefault;
type Flags = record Debug: boolean := false; Verbose: boolean := true; end;
begin
  var F: Flags := record Debug := true; end;
  Std.Console.WriteLn(F.Debug);
  Std.Console.WriteLn(F.Verbose)
end.",
    );
    assert_eq!(out.lines, vec!["true", "true"]);
}

#[test]
fn record_default_string_field() {
    let out = compile_and_run(
        "\
program RecStrDefault;
type Person = record Name: string := 'Anonymous'; Age: integer; end;
begin
  var P: Person := record Age := 25; end;
  Std.Console.WriteLn(P.Name);
  Std.Console.WriteLn(P.Age)
end.",
    );
    assert_eq!(out.lines, vec!["Anonymous", "25"]);
}

#[test]
fn record_default_not_used_when_field_explicitly_provided() {
    // All fields provided; defaults must not override explicit values.
    let out = compile_and_run(
        "\
program RecFull;
type Point = record X: integer := -1; Y: integer := -1; end;
begin
  var P: Point := record X := 10; Y := 20; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}

#[test]
fn record_default_required_and_defaulted_mix() {
    // Required fields must still be provided; defaulted fields may be omitted.
    let out = compile_and_run(
        "\
program RecMix;
type Vertex = record Id: integer; X: integer := 0; Y: integer := 0; end;
begin
  var V: Vertex := record Id := 7; end;
  Std.Console.WriteLn(V.Id);
  Std.Console.WriteLn(V.X);
  Std.Console.WriteLn(V.Y)
end.",
    );
    assert_eq!(out.lines, vec!["7", "0", "0"]);
}

#[test]
fn record_default_negative_integer_value() {
    let out = compile_and_run(
        "\
program RecNegDefault;
type Offset = record Dx: integer := -10; Dy: integer := -20; end;
begin
  var O: Offset := record end;
  Std.Console.WriteLn(O.Dx);
  Std.Console.WriteLn(O.Dy)
end.",
    );
    assert_eq!(out.lines, vec!["-10", "-20"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE: MISSING REQUIRED FIELDS
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_missing_required_field_with_defaults_present() {
    // X is required, Y has a default. Omitting X must be an error.
    let err = compile_err(
        "\
program MissingRequired;
type Point = record X: integer; Y: integer := 0; end;
begin
  var P: Point := record Y := 5; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD);
}

#[test]
fn record_missing_multiple_required_fields_error() {
    let err = compile_err(
        "\
program MissingMultiple;
type Rect = record X: integer; Y: integer; W: integer := 100; H: integer := 50; end;
begin
  var R: Rect := record W := 200; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE: TYPE ERRORS
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_wrong_type_for_defaulted_field() {
    // Providing the wrong type for a field that has a default must still be caught.
    let err = compile_err(
        "\
program WrongTypeDefault;
type Point = record X: integer := 0; Y: integer := 0; end;
begin
  var P: Point := record X := 'hello'; Y := 2; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}
