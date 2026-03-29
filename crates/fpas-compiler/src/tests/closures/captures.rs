use super::*;
#[test]
fn lambda_captures_enclosing_variable() {
    let out = compile_and_run(
        "\
program ClosureCapture;
uses Std.Console;
function MakeAdder(N: integer): function(X: integer): integer;
begin
  return function(X: integer): integer
  begin
    return X + N
  end
end;
begin
  var Add5: function(X: integer): integer := MakeAdder(5);
  WriteLn(Add5(10))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}
#[test]
fn multiple_closures_from_same_factory() {
    let out = compile_and_run(
        "\
program MultiClosure;
uses Std.Console;
function MakeMultiplier(Factor: integer): function(X: integer): integer;
begin
  return function(X: integer): integer
  begin
    return X * Factor
  end
end;
begin
  var Times2: function(X: integer): integer := MakeMultiplier(2);
  var Times5: function(X: integer): integer := MakeMultiplier(5);
  WriteLn(Times2(3));
  WriteLn(Times5(3))
end.",
    );
    assert_eq!(out.lines, vec!["6", "15"]);
}
#[test]
fn closure_captures_string() {
    let out = compile_and_run(
        "\
program ClosureStr;
uses Std.Console;
function MakeGreeter(Greeting: string): function(Name: string): string;
begin
  return function(Name: string): string
  begin
    return Greeting + ' ' + Name
  end
end;
begin
  var Hello: function(Name: string): string := MakeGreeter('Hello');
  WriteLn(Hello('World'))
end.",
    );
    assert_eq!(out.lines, vec!["Hello World"]);
}

#[test]
fn closure_captures_by_value_at_creation_time() {
    let out = compile_and_run(
        "\
program ClosureByValue;
uses Std.Console;
function BuildReader(): function(): integer;
begin
  mutable var N: integer := 1;
  var ReadN: function(): integer :=
    function(): integer
    begin
      return N
    end;
  N := 2;
  return ReadN
end;
begin
  var Reader: function(): integer := BuildReader();
  WriteLn(Reader())
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn closure_captures_boolean() {
    let out = compile_and_run(
        "\
program ClosureBool;
uses Std.Console;
function MakeChecker(Negate: boolean): function(X: boolean): boolean;
begin
  return function(X: boolean): boolean
  begin
    if Negate then
      return not X
    else
      return X
  end
end;
begin
  var Inverter: function(X: boolean): boolean := MakeChecker(true);
  WriteLn(Inverter(true));
  WriteLn(Inverter(false))
end.",
    );
    assert_eq!(out.lines, vec!["false", "true"]);
}

#[test]
fn closure_captures_multiple_variables() {
    let out = compile_and_run(
        "\
program ClosureMultiCapture;
uses Std.Console;
function MakeLinear(A: integer; B: integer): function(X: integer): integer;
begin
  return function(X: integer): integer
  begin
    return A * X + B
  end
end;
begin
  var F: function(X: integer): integer := MakeLinear(3, 7);
  WriteLn(F(10))
end.",
    );
    assert_eq!(out.lines, vec!["37"]);
}

#[test]
fn nested_closure_returns_closure() {
    let out = compile_and_run(
        "\
program NestedClosure;
uses Std.Console;
function MakeOuter(A: integer): function(B: integer): function(X: integer): integer;
begin
  return function(B: integer): function(X: integer): integer
  begin
    return function(X: integer): integer
    begin
      return A + B + X
    end
  end
end;
begin
  var Middle: function(B: integer): function(X: integer): integer := MakeOuter(100);
  var Inner: function(X: integer): integer := Middle(20);
  WriteLn(Inner(3))
end.",
    );
    assert_eq!(out.lines, vec!["123"]);
}
