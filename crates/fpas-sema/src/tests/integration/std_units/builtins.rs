use super::check_errors;

#[test]
fn unwrap_rejects_non_containers_without_cascading_argument_errors() {
    for operation in ["Unwrap", "UnwrapOr"] {
        for argument in ["42", "MissingValue"] {
            let fallback = if operation == "UnwrapOr" { "0" } else { "" };
            let errors = check_errors(&format!(
                "program T; begin const N: integer := ({argument}).{operation}({fallback}); end."
            ));
            assert_eq!(errors.len(), 1, "{errors:#?}");
            assert!(
                errors[0].message.contains(if argument == "42" {
                    "has no dot operation"
                } else {
                    "MissingValue"
                }),
                "{errors:#?}"
            );
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
  Std.Math.Sqrt(1.0, 2.0);
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
  const N: integer := Std.Conv.StrToInt(42);
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
fn native_format_has_no_free_factory_form() {
    let errors = check_errors("program T; begin const S: string := Format(); end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("Value.Format")),
        "{errors:#?}"
    );
}

#[test]
fn native_format_requires_string_receiver() {
    let errors = check_errors("program T; begin const S: string := (42).Format(); end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0].message.contains("has no dot operation"),
        "{errors:#?}"
    );
}

#[test]
fn std_array_push_requires_mutable_array() {
    let errs = check_errors(
        "program T;\n\nbegin\n  const A: array of integer := [1];\n  A.Push(2);\nend.",
    );
    assert!(errs.iter().any(|e| e.message.contains("var")), "{errs:#?}");
}

#[test]
fn std_dict_merge_requires_matching_rhs_dict_type() {
    let errs = check_errors(
        "program T;\n\nbegin\n  const M: dict of integer to integer := [1: 10].Merge(['x': true]);\nend.",
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
        "program T;\n\nbegin\n  const M: dict of integer to integer := [1: 10].Merge(42);\nend.",
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
        "program T;\n\nbegin\n  const V: Option of integer := ['Alice': 1].Get(42);\nend.",
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
        "program T;\n\nfunction WrongReturn(X: integer): integer;\nbegin\n  return X;\nend function;\nbegin\n  const V: Option of integer := [1, 2, 3].Find(WrongReturn);\nend.",
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
        "program T;\n\nfunction NotAProcedure(X: integer): integer;\nbegin\n  return X;\nend function;\nbegin\n  [1, 2, 3].ForEach(NotAProcedure);\nend.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("second argument must be a procedure")),
        "{errs:#?}"
    );
}
