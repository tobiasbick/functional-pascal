use super::*;

#[test]
fn std_math_pi_sqrt_abs_min_max() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Math.Sqrt(16.0));
  Std.Console.WriteLn(Std.Math.Abs(-5));
  Std.Console.WriteLn(Std.Math.Min(2, 10));
  Std.Console.WriteLn(Std.Math.Round(Std.Math.Pi));
  Std.Console.WriteLn(Std.Math.Pow(2.0, 3.0));
  Std.Console.WriteLn(Std.Math.Floor(2.9));
  Std.Console.WriteLn(Std.Math.Ceil(2.1));
  Std.Console.WriteLn(Std.Math.Sin(0.0));
  Std.Console.WriteLn(Std.Math.Cos(0.0));
  Std.Console.WriteLn(Std.Math.Log(1.0))
end.",
    );
    assert_eq!(
        out.lines,
        vec!["4", "5", "2", "3", "8", "2", "3", "0", "1", "0"]
    );
}

#[test]
fn std_sqrt_negative_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var R: real := Std.Math.Sqrt(-1.0)
end.",
    );
    assert!(msg.contains("Sqrt") || msg.contains("negative"), "{msg}");
}

#[test]
fn std_log_non_positive_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var R: real := Std.Math.Log(0.0)
end.",
    );
    assert!(msg.contains("Log") || msg.contains("positive"), "{msg}");
}

#[test]
fn std_floor_rejects_non_finite_results() {
    let msg = compile_run_err(
        "\
program T;
begin
  var N: integer := Std.Math.Floor(Std.Math.Exp(1000.0))
end.",
    );
    assert!(
        msg.contains("Floor result") || msg.contains("integer range"),
        "{msg}"
    );
}

#[test]
fn std_math_negative_rounding_variants() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Math.Floor(-3.2));
  Std.Console.WriteLn(Std.Math.Ceil(-3.2));
  Std.Console.WriteLn(Std.Math.Trunc(-3.7))
end.",
    );
    assert_eq!(out.lines, vec!["-4", "-3", "-3"]);
}

#[test]
fn std_trunc_rejects_negative_non_finite_results() {
    let msg = compile_run_err(
        "\
program T;
begin
  var N: integer := Std.Math.Trunc(-Std.Math.Exp(1000.0))
end.",
    );
    assert!(
        msg.contains("Trunc result") || msg.contains("integer range"),
        "{msg}"
    );
}
