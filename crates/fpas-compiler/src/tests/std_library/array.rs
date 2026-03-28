use super::*;

#[test]
fn std_array_sort_push_pop_slice() {
    let out = compile_and_run(
        "\
program T;
begin
  mutable var A: array of integer := [3, 1, 2];
  Std.Console.WriteLn(Std.Array.Length(A));
  Std.Array.Push(A, 4);
  Std.Console.WriteLn(Std.Array.Length(A));
  var Last: integer := Std.Array.Pop(A);
  Std.Console.WriteLn(Last);
  var B: array of integer := Std.Array.Sort(A);
  Std.Console.WriteLn(Std.Array.IndexOf(B, 2));
  var C: array of integer := Std.Array.Slice(B, 1, 2);
  Std.Console.WriteLn(Std.Array.Length(C));
  var R: array of integer := Std.Array.Reverse(B);
  Std.Console.WriteLn(Std.Array.Contains(B, 2));
  Std.Console.WriteLn(Std.Array.Contains(B, 99));
  Std.Console.WriteLn(Std.Array.IndexOf(R, 3))
end.",
    );
    assert_eq!(
        out.lines,
        vec!["3", "4", "4", "1", "2", "true", "false", "0"]
    );
}

#[test]
fn std_array_index_of_literal_array() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Array.IndexOf([1, 2, 3], 2))
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn std_array_index_of_var_from_literal() {
    let out = compile_and_run(
        "\
program T;
begin
  var B: array of integer := [1, 2, 3];
  Std.Console.WriteLn(Std.Array.IndexOf(B, 2))
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn std_array_pop_empty_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  mutable var A: array of integer := [1];
  Std.Array.Pop(A);
  Std.Array.Pop(A)
end.",
    );
    assert!(msg.contains("empty") || msg.contains("Pop"), "{msg}");
}

#[test]
fn std_slice_out_of_range_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var A: array of integer := [1, 2];
  var B: array of integer := Std.Array.Slice(A, 0, 5)
end.",
    );
    assert!(msg.contains("Slice") || msg.contains("range"), "{msg}");
}
