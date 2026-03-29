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

#[test]
fn std_dict_merge_requires_matching_rhs_dict_type() {
    let errs = check_errors(
        "\
program T;
uses Std.Dict;
begin
  var M: dict of integer to integer := Std.Dict.Merge([1: 10], ['x': true])
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("same key and value types")),
        "{errs:#?}"
    );
}

#[test]
fn std_dict_merge_requires_dict_rhs() {
    let errs = check_errors(
        "\
program T;
uses Std.Dict;
begin
  var M: dict of integer to integer := Std.Dict.Merge([1: 10], 42)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("dict as second argument")),
        "{errs:#?}"
    );
}

#[test]
fn std_dict_get_requires_matching_key_type() {
    let errs = check_errors(
        "\
program T;
uses Std.Dict;
begin
  var V: Option of integer := Std.Dict.Get(['Alice': 1], 42)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Type mismatch in dict key")),
        "{errs:#?}"
    );
}

#[test]
fn std_array_find_requires_boolean_callback_result() {
    let errs = check_errors(
        "\
program T;
uses Std.Array;
begin
  var V: Option of integer := Std.Array.Find([1, 2, 3],
    function(X: integer): integer begin return X end)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Type mismatch in callback return type")),
        "{errs:#?}"
    );
}

#[test]
fn std_array_for_each_requires_procedure_callback() {
    let errs = check_errors(
        "\
program T;
uses Std.Array;
begin
  Std.Array.ForEach([1, 2, 3],
    function(X: integer): integer begin return X end)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("second argument must be a procedure")),
        "{errs:#?}"
    );
}
