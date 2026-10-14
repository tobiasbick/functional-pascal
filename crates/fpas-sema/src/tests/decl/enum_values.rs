//! Enum identity restrictions and actionable declaration diagnostics.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_BACKING_VALUE_IN_DATA_ENUM, SEMA_DUPLICATE_ENUM_BACKING_VALUE};

#[test]
fn enum_values_reject_collisions_across_explicit_and_implicit_numbering() {
    for members in [
        "First = 1; Second = 1;",
        "First = 1; Middle = 5; Second = 1;",
        "First = 1; Restart = 0; Second;",
        "First; Second = 0;",
        "First = 9223372036854775807; Second = 9223372036854775807;",
        "First = $7FFFFFFFFFFFFFFF; Second = 9223372036854775807;",
    ] {
        let source = format!("program T; type Phase = enum {members} end enum; begin end.");
        let errors = check_errors(&source);
        assert_eq!(errors.len(), 1, "{members}: {errors:#?}");
        let error = &errors[0];
        assert_eq!(error.code, SEMA_DUPLICATE_ENUM_BACKING_VALUE);
        assert!(error.message.contains("Phase.First"), "{error:#?}");
        assert!(error.message.contains("Phase.Second"), "{error:#?}");
        assert!(
            error
                .help
                .as_ref()
                .is_some_and(|help| help.contains("unused")),
            "{error:#?}"
        );
        assert!(
            error
                .span
                .is_some_and(|span| source[span.offset()..span.end()].starts_with("Second")),
            "diagnostic must identify the conflicting declaration: {error:#?}"
        );
    }
}

#[test]
fn enum_values_reject_backing_annotations_anywhere_in_a_data_enum() {
    for (parameters, members) in [
        ("", "Empty = 7; Number(Value: integer);"),
        ("", "Number(Value: integer); Empty = 7;"),
        (" of T", "Empty = 7; Number(Value: T);"),
        (" of T", "Number(Value: T); Empty = 7;"),
    ] {
        let source =
            format!("program T; type Token{parameters} = enum {members} end enum; begin end.");
        let errors = check_errors(&source);
        assert_eq!(errors.len(), 1, "{members}: {errors:#?}");
        let error = &errors[0];
        assert_eq!(error.code, SEMA_BACKING_VALUE_IN_DATA_ENUM);
        assert!(error.message.contains("Token.Empty"), "{error:#?}");
        assert!(
            error
                .help
                .as_ref()
                .is_some_and(|help| help.contains("Empty;")),
            "{error:#?}"
        );
        assert!(
            error
                .span
                .is_some_and(|span| source[span.offset()..span.end()].starts_with("Empty"))
        );
    }
}

#[test]
fn enum_values_check_the_full_signed_backing_domain_in_the_ast() {
    // The source grammar has unsigned literals; the public AST stores signed values.
    for (first, restart, second) in [(i64::MIN, Some(0), Some(i64::MIN)), (-1, Some(-2), None)] {
        let (mut program, errors) = fpas_parser::parse(
            "program T; type Phase = enum First = 0; Restart = 1; Second; end enum; begin end.",
        );
        assert!(errors.is_empty(), "{errors:#?}");
        let fpas_parser::Decl::TypeDef(definition) = &mut program.declarations[0] else {
            panic!("enum declaration");
        };
        let fpas_parser::TypeBody::Enum(enumeration) = &mut definition.body else {
            panic!("enum body");
        };
        enumeration.members[0].value = Some(first);
        enumeration.members[1].value = restart;
        enumeration.members[2].value = second;
        let errors = crate::analyze(&program);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_DUPLICATE_ENUM_BACKING_VALUE);
        assert!(
            errors[0].message.contains(&first.to_string()),
            "{errors:#?}"
        );
    }
}

#[test]
fn enum_values_preserve_unique_sequences_and_fieldless_data_alternatives() {
    check_ok(
        "program T;
        type Phase = enum First; Next; Restart = 10; AfterRestart; Last = 9223372036854775807; end enum;
        type Token = enum Empty; Number(Value: integer); end enum;
        type GenericToken of T = enum Empty; Present(Value: T); end enum;
        begin
          const Value: GenericToken of integer := GenericToken.Empty;
        end.",
    );
}
