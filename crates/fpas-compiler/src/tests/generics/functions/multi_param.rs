use super::*;

#[test]
fn generic_function_two_type_params() {
    let out = compile_and_run(
        "\
program GenericTwoParams;
uses Std.Console;

function First<A, B>(X: A; Y: B): A;
begin
  return X
end;

begin
  WriteLn(First(10, 'ignored'))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn generic_function_second_of_two_params() {
    let out = compile_and_run(
        "\
program GenericSecond;
uses Std.Console;

function Second<A, B>(X: A; Y: B): B;
begin
  return Y
end;

begin
  WriteLn(Second(10, 'kept'))
end.",
    );
    assert_eq!(out.lines, vec!["kept"]);
}

#[test]
fn generic_procedure() {
    let out = compile_and_run(
        "\
program GenericProc;
uses Std.Console;

procedure PrintValue<T>(Value: T);
begin
  WriteLn(Value)
end;

begin
  PrintValue(99);
  PrintValue('world')
end.",
    );
    assert_eq!(out.lines, vec!["99", "world"]);
}
