//! Distinct type declarations, underlying types, and constant conversions.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use fpas_diagnostics::codes::{SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH};

use super::{check_errors, check_ok};

fn assert_single_error(source: &str, code: fpas_diagnostics::DiagnosticCode, message: &str) {
    let errors = check_errors(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, code, "{errors:#?}");
    assert!(errors[0].message.contains(message), "{errors:#?}");
}

#[test]
fn scalar_underlying_types_and_aliases_declare_distinct_types() {
    check_ok(
        "program T;
         type Raw = integer;
         type UserId = distinct integer;
         type Account = distinct Raw;
         type Price = distinct real;
         type Name = distinct string;
         type Flag = distinct boolean;
         type SameUser = UserId;
         begin
           const Id: SameUser := UserId(1);
           const A: Account := Account(2);
           const P: Price := Price(2.5);
           const N: Name := Name('Ada');
           const F: Flag := Flag(true);
         end.",
    );
}

#[test]
fn distinct_types_may_be_declared_after_their_use_and_before_their_underlying_alias() {
    check_ok(
        "program T;
         function Make(): UserId; begin return UserId(1); end function;
         type UserId = distinct Raw;
         type Raw = integer;
         begin end.",
    );
}

#[test]
fn plain_aliases_remain_interchangeable_with_their_target() {
    check_ok(
        "program T;
         type Count = integer;
         begin
           const C: Count := 1;
           const I: integer := C + 1;
         end.",
    );
}

#[test]
fn non_scalar_underlying_types_are_rejected() {
    for (underlying, reason) in [
        ("Point", "already has its own type identity"),
        ("Color", "already has its own type identity"),
        ("UserId", "is already a distinct type"),
        ("array of integer", "is not a scalar type"),
        ("dict of string to integer", "is not a scalar type"),
        ("Option of integer", "is not a scalar type"),
        ("function(X: integer): integer", "is not a scalar type"),
    ] {
        let errors = check_errors(&format!(
            "program T;
             type Point = record X: integer; end record;
             type Color = enum Red; end enum;
             type UserId = distinct integer;
             type Bad = distinct {underlying};
             begin end."
        ));
        assert_eq!(errors.len(), 1, "{underlying}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH);
        assert!(
            errors[0].message.contains(reason) && errors[0].message.contains("`Bad`"),
            "{underlying}: {errors:#?}"
        );
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("`integer`, `real`, `string`, or `boolean`")),
            "{underlying}: {errors:#?}"
        );
    }
}

#[test]
fn conversions_of_constants_are_compile_time_constants() {
    check_ok(
        "program T;
         type UserId = distinct integer;
         const Admin: UserId := UserId(1);
         const Raw: integer := integer(Admin);
         begin
           case Raw of
             when Raw: null;
             else null;
           end case;
         end.",
    );
}

#[test]
fn conversions_of_computed_values_are_computed() {
    assert_single_error(
        "program T;
         type UserId = distinct integer;
         function Load(): integer; begin return 1; end function;
         const Computed: UserId := UserId(Load());
         const Raw: integer := integer(Computed);
         begin
           case 1 of
             when Raw: null;
             else null;
           end case;
         end.",
        SEMA_NON_CONSTANT_EXPRESSION,
        "binding `Raw`",
    );
}

#[test]
fn distinct_record_field_defaults_use_explicit_conversions() {
    check_ok(
        "program T;
         type UserId = distinct integer;
         type User = record Id: UserId := UserId(0); Name: string; end record;
         begin
           const U: User := User(Name := 'Ada');
           const Raw: integer := integer(U.Id);
         end.",
    );
}
