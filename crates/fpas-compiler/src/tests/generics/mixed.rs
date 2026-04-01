use super::*;

// =============================================================
// MIXED: generic functions with concrete parameters
// =============================================================

#[test]
fn generic_function_with_generic_param_and_concrete() {
    let out = compile_and_run(
        "\
program GenericMixedParams;
uses Std.Console, Std.Conv;

function Describe<T>(Value: T; Label: string): string;
begin
  return Label + ': done'
end;

begin
  WriteLn(Describe(42, 'number'))
end.",
    );
    assert_eq!(out.lines, vec!["number: done"]);
}
