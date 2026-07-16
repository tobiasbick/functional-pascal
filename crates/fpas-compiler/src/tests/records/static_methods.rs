use super::*;

#[test]
fn static_record_function_create_and_use() {
    let out = compile_and_run(
        "\
program StaticCreate;
type Point = record
  X: integer;
  Y: integer;
  static function Create(X: integer; Y: integer): Point;
  begin
    return record
      X := X;
      Y := Y;
    end
  end;
  function Sum(Self: Point): integer;
  begin
    return Self.X + Self.Y
  end;
end;
begin
  var P: Point := Point.Create(3, 7);
  Std.Console.WriteLn(P.Sum())
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn static_record_function_no_receiver_on_stack() {
    let out = compile_and_run(
        "\
program StaticArity;
type Counter = record
  Value: integer;
  static function FromValue(V: integer): Counter;
  begin
    return record Value := V; end
  end;
  static function Zero(): Counter;
  begin
    return Counter.FromValue(0)
  end;
end;
begin
  var C: Counter := Counter.Zero();
  Std.Console.WriteLn(C.Value)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn static_record_function_case_insensitive_at_runtime() {
    let out = compile_and_run(
        "\
program StaticCase;
type Point = record
  X: integer;
  Y: integer;
  static function Create(X: integer; Y: integer): Point;
  begin
    return record X := X; Y := Y; end
  end;
end;
begin
  var P: Point := point.create(4, 5);
  Std.Console.WriteLn(P.X + P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["9"]);
}

#[test]
fn static_record_function_via_type_alias() {
    let out = compile_and_run(
        "\
program StaticAlias;
type Point = record
  X: integer;
  Y: integer;
  static function Create(X: integer; Y: integer): Point;
  begin
    return record X := X; Y := Y; end
  end;
end;
type Pt = Point;
begin
  var P: Pt := Pt.Create(2, 8);
  Std.Console.WriteLn(P.X * P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["16"]);
}

#[test]
fn static_record_function_returns_non_record() {
    let out = compile_and_run(
        "\
program StaticOtherReturn;
type Point = record
  X: integer;
  Y: integer;
  static function Dot(A: Point; B: Point): integer;
  begin
    return A.X * B.X + A.Y * B.Y
  end;
end;
begin
  var A: Point := record X := 2; Y := 3; end;
  var B: Point := record X := 4; Y := 5; end;
  Std.Console.WriteLn(Point.Dot(A, B))
end.",
    );
    assert_eq!(out.lines, vec!["23"]);
}

#[test]
fn static_generic_record_function() {
    let out = compile_and_run(
        "\
program StaticGeneric;
type Id = record
  static function Identity<T>(V: T): T;
  begin
    return V
  end;
end;
begin
  Std.Console.WriteLn(Id.Identity(99))
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}
