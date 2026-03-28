use super::*;

#[test]
fn generic_function_with_array_param() {
    let out = compile_and_run(
        "\
program GenericArray;
uses Std.Console, Std.Array;

function FirstElement<T>(Items: array of T): T;
begin
  return Items[0]
end;

begin
  WriteLn(FirstElement([10, 20, 30]))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn generic_function_returning_array() {
    let out = compile_and_run(
        "\
program GenericRetArr;
uses Std.Console, Std.Array, Std.Conv;

function Wrap<T>(V: T): array of T;
begin
  return [V]
end;

begin
  var A: array of integer := Wrap(42);
  WriteLn(IntToStr(Std.Array.Length(A)));
  WriteLn(A[0])
end.",
    );
    assert_eq!(out.lines, vec!["1", "42"]);
}
