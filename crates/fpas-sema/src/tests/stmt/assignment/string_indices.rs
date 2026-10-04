//! String index reads must never become writable character storage.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

#[test]
fn string_index_assignment_reports_the_index_and_replacement_hint() {
    let errors = check_errors(
        r#"program T;
begin
   var Text: string := 'abc';
  Text[0] := 'z';
end program;"#,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
    assert_eq!(errors[0].message, "String indices are read-only");
    let span = errors[0].span.expect("index source span");
    assert_eq!((span.line(), span.column()), (4, 7));
    assert!(errors[0].help.as_deref().unwrap().contains("whole string"));
}

#[test]
fn nested_string_index_assignment_is_rejected_through_collections_and_aliases() {
    let declarations = "
        type TextAlias = string;
        type Holder of (T) = record Value: T; end record;
        type TextHolder = Holder of (TextAlias);";
    for (binding, target) in [
        ("Text: TextAlias := 'abc'", "Text[0]"),
        ("Text: string := 'abc'", "Text[0][0]"),
        ("Items: array of (string) := ['abc']", "Items[0][0]"),
        (
            "Items: dict of (string, string) := ['key': 'abc']",
            "Items['key'][0]",
        ),
        (
            "Item: TextHolder := TextHolder(Value := 'abc')",
            "Item.Value[0]",
        ),
        (
            "Items: dict of (string, array of (TextHolder)) := ['key': [TextHolder(Value := 'abc')]]",
            "Items['key'][0].Value[0]",
        ),
    ] {
        let source = format!(
            r#"program T; {declarations} begin  var {binding}; {target} := 'z'; end program;"#
        );
        let errors = check_errors(&source);
        assert_eq!(errors.len(), 1, "{target}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT, "{target}");
        assert_eq!(
            errors[0].message, "String indices are read-only",
            "{target}"
        );
    }
}

#[test]
fn string_index_assignment_is_rejected_for_globals_parameters_and_captures() {
    for source in [
        r#"program T;  var Text: string := 'abc'; begin Text[0] := 'z'; end program;"#,
        "program T; procedure Change(Text: string); begin Text[0] := 'z'; end procedure; begin null; end program;",
        r#"program T; begin  var Text: string := 'abc'; const Change: procedure() := procedure() begin Text[0] := 'z'; end procedure; Change(); end program;"#,
    ] {
        let errors = check_errors(source);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
        assert_eq!(errors[0].message, "String indices are read-only");
    }
}

#[test]
fn immutable_string_roots_report_one_read_only_index_error() {
    for declaration in ["var Text: string := 'abc';", "const Text: string := 'abc';"] {
        let errors = check_errors(&format!(
            "program T; {declaration} begin Text[0] := 'z'; end program;"
        ));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
        assert_eq!(errors[0].message, "String indices are read-only");
    }
}

#[test]
fn invalid_target_paths_keep_their_original_diagnostic() {
    for (target, code) in [
        ("Text[true]", SEMA_TYPE_MISMATCH),
        ("Text[MissingIndex()]", SEMA_UNKNOWN_NAME),
        ("Missing[0]", SEMA_UNKNOWN_NAME),
        ("Text[0].Missing", SEMA_TYPE_MISMATCH),
    ] {
        let errors = check_errors(&format!(
            r#"program T; begin  var Text: string := 'abc'; {target} := 'z'; end program;"#
        ));
        assert_eq!(errors.len(), 1, "{target}: {errors:#?}");
        assert_eq!(errors[0].code, code, "{target}");
    }
}

#[test]
fn string_index_assignment_still_checks_the_value_once() {
    let errors = check_errors(
        r#"program T; begin  var Text: string := 'abc'; Text[0] := MissingValue(); end program;"#,
    );
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME);
    assert_eq!(errors[1].code, SEMA_IMMUTABLE_ASSIGNMENT);
}

#[test]
fn string_reads_and_whole_string_replacements_remain_valid() {
    check_ok(
        r#"program T;
        type Holder of (T) = record Value: T; end record;
        type TextHolder = Holder of (string);
        begin
           var Text: string := 'abc';
           var Items: array of (string) := ['abc'];
           var Mapping: dict of (string, string) := ['key': 'abc'];
           var Nested: dict of (string, array of (TextHolder)) := ['key': [TextHolder(Value := 'abc')]];
          Text := 'ä😀';
          Items[0] := Text;
          Mapping['key'] := Text;
          Nested['key'][0].Value := Text;
          const Character: string := Nested['key'][0].Value[1];
          const Again: string := Mapping['key'][0][0];
        end program;"#,
    );
}
