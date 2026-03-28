use super::super::*;

#[test]
fn short_array_sort_push_pop() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [3, 1, 2];
  WriteLn(Length(A));
  Push(A, 4);
  WriteLn(Length(A));
  var Last: integer := Pop(A);
  WriteLn(Last);
  var B: array of integer := Sort(A);
  WriteLn(IndexOf(B, 2));
  var C: array of integer := Slice(B, 1, 2);
  WriteLn(Length(C));
  WriteLn(Contains(B, 2));
  WriteLn(Contains(B, 99));
  var R: array of integer := Reverse(B);
  WriteLn(IndexOf(R, 3))
end.",
    );
    assert_eq!(
        out.lines,
        vec!["3", "4", "4", "1", "2", "true", "false", "0"]
    );
}
