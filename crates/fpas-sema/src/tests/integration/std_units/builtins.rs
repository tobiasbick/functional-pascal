use super::check_errors;

#[test]
fn unwrap_rejects_non_containers_without_cascading_argument_errors() {
    for namespace in ["Options", "Results"] {
        for function in ["Unwrap", "UnwrapOr"] {
            for argument in ["42", "MissingValue"] {
                let fallback = if function == "UnwrapOr" { ", 0" } else { "" };
                let errs = check_errors(&format!(
                    "program T;\nuses Std.{namespace} as {namespace};\nbegin\n  var N: integer := {namespace}.{function}({argument}{fallback});\nend program;"
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
        r#"program T;
uses Std.Math as Math;
begin
  Math.Sqrt(1.0, 2.0);
end program;"#,
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
        r#"program T;
uses Std.Conv as Conv;
begin
  var N: integer := Conv.StrToInt(42);
end program;"#,
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
        r#"program T;
uses Std.Str as Str;
begin
  var S: string := Str.Format();
end program;"#,
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
        r#"program T;
uses Std.Str as Str;
begin
  var S: string := Str.Format(42);
end program;"#,
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
        r#"program T;
uses Std.Arrays as Arrays;
begin
  var A: array of (integer) := [1];
  Arrays.Push(A, 2);
end program;"#,
    );
    assert!(
        errs.iter().any(|e| e.message.contains("mutable var")),
        "{errs:#?}"
    );
}

#[test]
fn std_dict_merge_requires_matching_rhs_dict_type() {
    let errs = check_errors(
        r#"program T;
uses Std.Dictionaries as Dictionaries;
begin
  var M: dict of (integer, integer) := Dictionaries.Merge([1: 10], ['x': true]);
end program;"#,
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
        r#"program T;
uses Std.Dictionaries as Dictionaries;
begin
  var M: dict of (integer, integer) := Dictionaries.Merge([1: 10], 42);
end program;"#,
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
        r#"program T;
uses Std.Dictionaries as Dictionaries;
begin
  var V: Option of (integer) := Dictionaries.Get(['Alice': 1], 42);
end program;"#,
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
        r#"program T;
uses Std.Arrays as Arrays;
function WrongReturn(X: integer): integer;
begin
  return X;
end function;
begin
  var V: Option of (integer) := Arrays.Find([1, 2, 3], WrongReturn);
end program;"#,
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
        r#"program T;
uses Std.Arrays as Arrays;
function NotAProcedure(X: integer): integer;
begin
  return X;
end function;
begin
  Arrays.ForEach([1, 2, 3], NotAProcedure);
end program;"#,
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("second argument must be a procedure")),
        "{errs:#?}"
    );
}
