/// Tests for the record `with` update expression (`base with Field := Value; … end`).
///
/// **Documentation:** [docs/pascal/05-types.md](docs/pascal/05-types.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE: BASIC UPDATE
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_with_single_field_update() {
    let out = compile_and_run(
        "\
program RecWith;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with X := 99; end;
  Std.Console.WriteLn(Q.X);
  Std.Console.WriteLn(Q.Y)
end.",
    );
    assert_eq!(out.lines, vec!["99", "2"]);
}

#[test]
fn record_with_multiple_fields_update() {
    let out = compile_and_run(
        "\
program RecWithMulti;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with X := 10; Y := 20; end;
  Std.Console.WriteLn(Q.X);
  Std.Console.WriteLn(Q.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}

#[test]
fn record_with_preserves_original() {
    // The original record must not be mutated by `with`.
    let out = compile_and_run(
        "\
program RecWithOrig;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with X := 99; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y);
  Std.Console.WriteLn(Q.X);
  Std.Console.WriteLn(Q.Y)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "99", "2"]);
}

#[test]
fn record_with_chained_updates() {
    let out = compile_and_run(
        "\
program RecWithChain;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 0; Y := 0; end;
  var Q: Point := (P with X := 5; end) with Y := 7; end;
  Std.Console.WriteLn(Q.X);
  Std.Console.WriteLn(Q.Y)
end.",
    );
    assert_eq!(out.lines, vec!["5", "7"]);
}

#[test]
fn record_with_on_function_return() {
    let out = compile_and_run(
        "\
program RecWithFunc;
type Point = record X: integer; Y: integer; end;
function Origin(): Point;
begin
  return record X := 0; Y := 0; end
end;
begin
  var P: Point := Origin() with X := 42; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["42", "0"]);
}

#[test]
fn record_with_string_field_update() {
    let out = compile_and_run(
        "\
program RecWithStr;
type Person = record Name: string; Age: integer; end;
begin
  var P: Person := record Name := 'Alice'; Age := 30; end;
  var Q: Person := P with Name := 'Bob'; end;
  Std.Console.WriteLn(Q.Name);
  Std.Console.WriteLn(Q.Age)
end.",
    );
    assert_eq!(out.lines, vec!["Bob", "30"]);
}

#[test]
fn record_with_result_passed_to_function() {
    // The result of `with` may be passed directly as a function argument.
    let out = compile_and_run(
        "\
program RecWithParam;
type Point = record X: integer; Y: integer; end;
function Sum(P: Point): integer;
begin
  return P.X + P.Y
end;
begin
  var P: Point := record X := 3; Y := 7; end;
  Std.Console.WriteLn(Sum(P with X := 10; end))
end.",
    );
    assert_eq!(out.lines, vec!["17"]);
}

// ═══════════════════════════════════════════════════════════════
// EDGE CASES: COMBINED WITH DEFAULTS
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_with_combined_with_defaults() {
    // A record built entirely from defaults can then be updated via `with`.
    let out = compile_and_run(
        "\
program RecWithDefaults;
type Config = record Host: string := 'localhost'; Port: integer := 8080; end;
begin
  var C: Config := record end;
  var D: Config := C with Port := 9000; end;
  Std.Console.WriteLn(D.Host);
  Std.Console.WriteLn(D.Port)
end.",
    );
    assert_eq!(out.lines, vec!["localhost", "9000"]);
}

#[test]
fn record_with_all_fields_overridden() {
    // Updating every field via `with` must produce the fully new values.
    let out = compile_and_run(
        "\
program RecWithAll;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with X := 10; Y := 20; end;
  Std.Console.WriteLn(Q.X);
  Std.Console.WriteLn(Q.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}

#[test]
fn record_with_only_unmodified_fields_preserved() {
    // Fields not mentioned in `with` must keep their original values.
    let out = compile_and_run(
        "\
program RecWithPartial;
type Color = record R: integer; G: integer; B: integer; end;
begin
  var C: Color := record R := 100; G := 150; B := 200; end;
  var D: Color := C with G := 0; end;
  Std.Console.WriteLn(D.R);
  Std.Console.WriteLn(D.G);
  Std.Console.WriteLn(D.B)
end.",
    );
    assert_eq!(out.lines, vec!["100", "0", "200"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE: UNKNOWN FIELD IN UPDATE
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_with_unknown_field_error() {
    let err = compile_err(
        "\
program RecWithBadField;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with Z := 3; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE: WRONG TYPE IN UPDATE
// ═══════════════════════════════════════════════════════════════

#[test]
fn record_with_wrong_type_error() {
    let err = compile_err(
        "\
program RecWithBadType;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 1; Y := 2; end;
  var Q: Point := P with X := 'hello'; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn record_with_on_non_record_error() {
    // Using `with` on a non-record value must be rejected.
    let err = compile_err(
        "\
program RecWithNonRec;
begin
  var X: integer := 42;
  var Y: integer := X with X := 1; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}
