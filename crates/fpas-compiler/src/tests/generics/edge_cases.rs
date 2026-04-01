use super::*;

// =============================================================
// EDGE CASES — generic functions
// =============================================================

#[test]
fn generic_single_letter_type_params() {
    let out = compile_and_run(
        "\
program SingleLetter;
uses Std.Console;

function Id<X>(V: X): X;
begin
  return V
end;

begin
  WriteLn(Id(7))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn generic_long_type_param_name() {
    let out = compile_and_run(
        "\
program LongParamName;
uses Std.Console;

function Id<Element>(V: Element): Element;
begin
  return V
end;

begin
  WriteLn(Id('test'))
end.",
    );
    assert_eq!(out.lines, vec!["test"]);
}

#[test]
fn generic_three_type_params() {
    let out = compile_and_run(
        "\
program ThreeParams;
uses Std.Console;

function Pick<A, B, C>(X: A; Y: B; Z: C): C;
begin
  return Z
end;

begin
  WriteLn(Pick(1, 'two', true))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}
