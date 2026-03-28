use super::check_errors;

#[test]
fn std_math_sqrt_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Math;
begin
  Std.Math.Sqrt(1.0, 2.0)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 1 argument")),
        "{errs:#?}"
    );
}

#[test]
fn std_conv_str_to_int_type_mismatch() {
    let errs = check_errors(
        "\
program T;
uses Std.Conv;
begin
  var N: integer := Std.Conv.StrToInt(42)
end.",
    );
    assert!(
        errs.iter().any(|e| {
            e.message.contains("string")
                || e.message.contains("type")
                || e.message.contains("argument")
        }),
        "{errs:#?}"
    );
}

#[test]
fn std_array_push_requires_mutable_array() {
    let errs = check_errors(
        "\
program T;
uses Std.Array;
begin
  var A: array of integer := [1];
  Std.Array.Push(A, 2)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("mutable var")),
        "{errs:#?}"
    );
}
