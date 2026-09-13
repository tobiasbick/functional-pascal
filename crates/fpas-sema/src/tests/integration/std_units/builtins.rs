use super::check_errors;

#[test]
fn unwrap_rejects_non_containers_without_cascading_argument_errors() {
    for namespace in ["Options", "Results"] {
        for function in ["Unwrap", "UnwrapOr"] {
            for argument in ["42", "MissingValue"] {
                let fallback = if function == "UnwrapOr" { ", 0" } else { "" };
                let errs = check_errors(&format!(
                    "program T;
uses Std.{namespace};
begin
  var N: integer := Std.{namespace}.{function}({argument}{fallback})
end."
                ));
                assert_eq!(errs.len(), 1, "{errs:#?}");
                if argument == "42" {
                    assert_eq!(errs[0].code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
                    assert!(errs[0].message.contains("first argument"), "{errs:#?}");
                } else {
                    assert!(errs[0].message.contains("MissingValue"), "{errs:#?}");
                    assert_ne!(errs[0].code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
                }
            }
        }
    }
}

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
fn std_str_format_requires_template_argument() {
    let errs = check_errors(
        "\
program T;
uses Std.Str;
begin
  var S: string := Std.Str.Format()
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT),
        "{errs:#?}"
    );
}

#[test]
fn std_str_format_checks_template_type() {
    let errs = check_errors(
        "\
program T;
uses Std.Str;
begin
  var S: string := Std.Str.Format(42)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "{errs:#?}"
    );
}

#[test]
fn std_array_push_requires_mutable_array() {
    let errs = check_errors(
        "\
program T;
uses Std.Arrays;
begin
  var A: array of integer := [1];
  Std.Arrays.Push(A, 2)
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
uses Std.Dictionaries;
begin
  var M: dict of integer to integer := Std.Dictionaries.Merge([1: 10], ['x': true])
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
uses Std.Dictionaries;
begin
  var M: dict of integer to integer := Std.Dictionaries.Merge([1: 10], 42)
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
uses Std.Dictionaries;
begin
  var V: Option of integer := Std.Dictionaries.Get(['Alice': 1], 42)
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
uses Std.Arrays;
function WrongReturn(X: integer): integer;
begin
  return X
end;
begin
  var V: Option of integer := Std.Arrays.Find([1, 2, 3], WrongReturn)
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
uses Std.Arrays;
function NotAProcedure(X: integer): integer;
begin
  return X
end;
begin
  Std.Arrays.ForEach([1, 2, 3], NotAProcedure)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("second argument must be a procedure")),
        "{errs:#?}"
    );
}
