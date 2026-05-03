use super::*;

#[test]
fn std_array_map_filter_reduce_and_flat_map() {
    let out = compile_and_run(
        "\
program T;
uses Std.Array, Std.Console;

function Double(X: integer): integer;
begin
  return X * 2
end;

function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;

function Add(Acc: integer; X: integer): integer;
begin
  return Acc + X
end;

function Expand(X: integer): array of integer;
begin
  return [X, X + 10]
end;

begin
  var Numbers: array of integer := [1, 2, 3];
  Std.Console.WriteLn(Std.Array.Map(Numbers, Double));
  Std.Console.WriteLn(Std.Array.Filter(Numbers, IsEven));
  Std.Console.WriteLn(Std.Array.Reduce(Numbers, 10, Add));
  Std.Console.WriteLn(Std.Array.FlatMap(Numbers, Expand))
end.",
    );

    assert_eq!(
        out.lines,
        vec!["[2, 4, 6]", "[2]", "16", "[1, 11, 2, 12, 3, 13]"]
    );
}

#[test]
fn std_array_find_find_index_any_and_all() {
    let out = compile_and_run(
        "\
program T;
uses Std.Array, Std.Console, Std.Option;

function AboveThree(X: integer): boolean;
begin
  return X > 3
end;

function Positive(X: integer): boolean;
begin
  return X > 0
end;

function Negative(X: integer): boolean;
begin
  return X < 0
end;

begin
  var Numbers: array of integer := [1, 4, 5];
  var Found: Option of integer := Std.Array.Find(Numbers, AboveThree);
  Std.Console.WriteLn(Std.Option.IsSome(Found));
  Std.Console.WriteLn(Std.Option.Unwrap(Found));
  Std.Console.WriteLn(Std.Array.FindIndex(Numbers, AboveThree));
  Std.Console.WriteLn(Std.Array.Any(Numbers, Negative));
  Std.Console.WriteLn(Std.Array.All(Numbers, Positive))
end.",
    );

    assert_eq!(out.lines, vec!["true", "4", "1", "false", "true"]);
}

#[test]
fn std_array_higher_order_empty_array_edges() {
    let out = compile_and_run(
        "\
program T;
uses Std.Array, Std.Console, Std.Option;

function Double(X: integer): integer;
begin
  return X * 2
end;

function AlwaysTrue(X: integer): boolean;
begin
  return true
end;

function Add(Acc: integer; X: integer): integer;
begin
  return Acc + X
end;

function EmptyExpand(X: integer): array of integer;
begin
  return []
end;

begin
  var Empty: array of integer := [];
  Std.Console.WriteLn(Std.Array.Length(Std.Array.Map(Empty, Double)));
  Std.Console.WriteLn(Std.Array.Length(Std.Array.Filter(Empty, AlwaysTrue)));
  Std.Console.WriteLn(Std.Array.Reduce(Empty, 42, Add));
  Std.Console.WriteLn(Std.Option.IsNone(Std.Array.Find(Empty, AlwaysTrue)));
  Std.Console.WriteLn(Std.Array.FindIndex(Empty, AlwaysTrue));
  Std.Console.WriteLn(Std.Array.Any(Empty, AlwaysTrue));
  Std.Console.WriteLn(Std.Array.All(Empty, AlwaysTrue));
  Std.Console.WriteLn(Std.Array.Length(Std.Array.FlatMap(Empty, EmptyExpand)))
end.",
    );

    assert_eq!(
        out.lines,
        vec!["0", "0", "42", "true", "-1", "false", "true", "0"]
    );
}

#[test]
fn std_array_for_each_invokes_procedure_for_each_element() {
    let out = compile_and_run(
        "\
program T;
uses Std.Array, Std.Console;

procedure PrintValue(X: integer);
begin
  Std.Console.WriteLn(X)
end;

begin
  Std.Array.ForEach([3, 1, 2], PrintValue)
end.",
    );

    assert_eq!(out.lines, vec!["3", "1", "2"]);
}
