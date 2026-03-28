use super::*;
#[test]
fn record_create_and_field_get() {
    let out = compile_and_run(
        "\
program RecGet;
type Point = record X: integer; Y: integer; end;
begin
  var P: Point := record X := 10; Y := 20; end;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}
#[test]
fn record_field_set() {
    let out = compile_and_run(
        "\
program RecSet;
type Point = record X: integer; Y: integer; end;
begin
  mutable var P: Point := record X := 1; Y := 2; end;
  P.X := 99;
  P.Y := 77;
  Std.Console.WriteLn(P.X);
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["99", "77"]);
}
#[test]
fn record_pass_to_function() {
    let out = compile_and_run(
        "\
program RecFunc;
type Point = record X: integer; Y: integer; end;
function Sum(P: Point): integer;
begin
  return P.X + P.Y
end;
begin
  var P: Point := record X := 3; Y := 7; end;
  Std.Console.WriteLn(Sum(P))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}
#[test]
fn record_with_string_fields() {
    let out = compile_and_run(
        "\
program RecStr;
type Person = record Name: string; Age: integer; end;
begin
  var P: Person := record Name := 'Alice'; Age := 30; end;
  Std.Console.WriteLn(P.Name);
  Std.Console.WriteLn(P.Age)
end.",
    );
    assert_eq!(out.lines, vec!["Alice", "30"]);
}

#[test]
fn record_with_boolean_and_real_fields() {
    let out = compile_and_run(
        "\
program RecMixed;
uses Std.Console;
type Config = record Active: boolean; Rate: real; end;
begin
  var C: Config := record Active := true; Rate := 3.14; end;
  WriteLn(C.Active);
  WriteLn(C.Rate)
end.",
    );
    assert_eq!(out.lines, vec!["true", "3.14"]);
}

#[test]
fn record_returned_from_function() {
    let out = compile_and_run(
        "\
program RecReturn;
uses Std.Console;
type Point = record X: integer; Y: integer; end;

function MakePoint(X: integer; Y: integer): Point;
begin
  return record X := X; Y := Y; end
end;

begin
  var P: Point := MakePoint(10, 20);
  WriteLn(P.X);
  WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}

#[test]
fn record_nested_field_access() {
    let out = compile_and_run(
        "\
program RecNested;
uses Std.Console;
type Inner = record V: integer; end;
type Outer = record I: Inner; end;
begin
  var O: Outer := record I := record V := 42; end; end;
  WriteLn(O.I.V)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn mutable_record_partial_field_update() {
    let out = compile_and_run(
        "\
program RecPartial;
uses Std.Console;
type Point = record X: integer; Y: integer; end;
begin
  mutable var P: Point := record X := 1; Y := 2; end;
  P.X := 99;
  WriteLn(P.X);
  WriteLn(P.Y)
end.",
    );
    // Y stays unchanged
    assert_eq!(out.lines, vec!["99", "2"]);
}
