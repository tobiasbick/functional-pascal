//! Normal resolution for names available to user declarations.
//!
//! **Documentation:** `docs/pascal/getting-started/keywords.md`

use super::super::{check_errors, check_ok};
use crate::analyze_with_types;
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;

#[test]
fn nil_and_assigned_without_declarations_are_unknown_names() {
    for expression in ["nil", "NiL", "Assigned(1)", "aSsIgNeD()"] {
        let errors = check_errors(&format!(
            "program T; begin const Value: integer := {expression}; end."
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME, "{errors:#?}");
    }
}

#[test]
fn ordinary_routine_names_resolve_case_insensitively() {
    for name in ["event", "nil", "read", "write", "Assigned"] {
        let source = format!(
            "program T;
             function {name}(Value: integer): integer;
             begin return Value; end function;
             begin const Value: integer := {}(1); end.",
            name.to_ascii_uppercase()
        );
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:#?}");
        let metadata = analyze_with_types(&program);
        assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
        assert!(metadata.intrinsic_calls.is_empty());
    }
}

#[test]
fn ordinary_record_members_follow_field_and_method_rules() {
    check_ok(
        "program T;
      type Event = record
        Nil: integer;
        function Read(Self: Event): integer;
        begin return Self.Nil; end function;
        function Write(Self: Event; Assigned: integer): Event;
        begin return Event(Nil := Assigned); end function;
      end record;
      begin
        var Value: Event := Event(Nil := 1);
        Value.NIL := 2;
        const Copy: Event := Value.WRITE(3);
        const ResultValue: integer := Copy.READ();
      end.",
    );
}
