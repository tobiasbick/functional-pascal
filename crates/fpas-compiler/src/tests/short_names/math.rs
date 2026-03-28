use super::super::*;

#[test]
fn short_math_sqrt_and_pi() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(16.0));
  WriteLn(Round(Pi))
end.",
    );
    assert_eq!(out.lines, vec!["4", "3"]);
}

#[test]
fn short_math_abs_min_max() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(-7));
  WriteLn(Min(3, 9));
  WriteLn(Max(3, 9))
end.",
    );
    assert_eq!(out.lines, vec!["7", "3", "9"]);
}

#[test]
fn short_math_pow_floor_ceil() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pow(2.0, 3.0));
  WriteLn(Floor(2.9));
  WriteLn(Ceil(2.1))
end.",
    );
    assert_eq!(out.lines, vec!["8", "2", "3"]);
}

#[test]
fn short_runtime_error_preserves_behavior() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Math;
begin
  var R: real := Sqrt(-1.0)
end.",
    );
    assert!(msg.contains("Sqrt") || msg.contains("negative"), "{msg}");
}
