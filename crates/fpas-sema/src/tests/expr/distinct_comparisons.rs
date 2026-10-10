//! Inherited comparisons, membership, and `Comparable` for distinct types.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

use super::{check_errors, check_ok};

const PRELUDE: &str = "program T;
  type UserId = distinct integer;
  type OrderId = distinct integer;
  type Name = distinct string;
  type Flag = distinct boolean;
  type User = record Id: UserId; end record;";

fn program(body: &str) -> String {
    format!("{PRELUDE}\nbegin\n{body}\nend.")
}

/// Asserts one type mismatch whose message and help contain the given fragments.
fn assert_rejected(body: &str, message: &str, help: &str) {
    let errors = check_errors(&program(body));
    assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
    assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{body}: {errors:#?}");
    assert!(errors[0].message.contains(message), "{body}: {errors:#?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|text| text.contains(help)),
        "{body}: {errors:#?}"
    );
}

#[test]
fn same_type_operands_inherit_equality_and_ordering() {
    check_ok(&program(
        "const A: UserId := UserId(1);
         const B: UserId := UserId(2);
         const Results: array of boolean := [A = B, A <> B, A < B, A > B, A <= B, A >= B];
         const Text: boolean := Name('a') < Name('b');
         const Truth: boolean := Flag(true) = Flag(false);
         const Records: boolean := User(Id := A) = User(Id := B);
         const Options: boolean := Some(A) = None;",
    ));
}

#[test]
fn mixed_domains_and_underlying_operands_are_rejected() {
    assert_rejected(
        "const Same: boolean := UserId(1) = OrderId(1);",
        "Cannot compare distinct type `UserId` with distinct type `OrderId`",
        "`integer(Left) = integer(Right)`",
    );
    assert_rejected(
        "const Same: boolean := UserId(1) < 1;",
        "Cannot compare distinct type `UserId` with `integer`",
        "`Value < UserId(42)`",
    );
    assert_rejected(
        "const Same: boolean := 'a' = Name('a');",
        "Cannot compare distinct type `Name` with `string`",
        "`string(Value)`",
    );
}

#[test]
fn membership_follows_equality_for_the_same_distinct_type() {
    check_ok(&program(
        "const Ids: array of UserId := [UserId(1), UserId(2)];
         const Lookup: dict of UserId to string := [UserId(1): 'one'];
         const InArray: boolean := UserId(2) in Ids;
         const InDict: boolean := UserId(1) in Lookup;
         const Value: string := Lookup[UserId(1)];",
    ));
    assert_rejected(
        "const Found: boolean := UserId(1) in [1, 2];",
        "Operator `in` requires `UserId` elements or keys, found `integer`",
        "`integer(Value)`",
    );
    assert_rejected(
        "const Found: boolean := OrderId(1) in [UserId(1)];",
        "requires `OrderId` elements or keys, found `UserId`",
        "collection of `OrderId` values",
    );
    assert_rejected(
        "const Found: boolean := Name('a') in Name('abc');",
        "Operator `in` is not defined for distinct type `Name`",
        "`string(Left) in string(Right)`",
    );
}

#[test]
fn logical_operators_remain_unavailable() {
    assert_rejected(
        "const Both: boolean := Flag(true) and Flag(false);",
        "Operator `and` is not defined for distinct type `Flag`",
        "`boolean(Left) and boolean(Right)`",
    );
}

#[test]
fn comparable_is_satisfied_but_numeric_and_printable_are_not() {
    check_ok(
        "program T;
         type UserId = distinct integer;
         function Max<T: Comparable>(Left: T; Right: T): T;
         begin if Left > Right then return Left; end if; return Right; end function;
         begin const Larger: UserId := Max(UserId(1), UserId(2)); end.",
    );
    let errors = check_errors(
        "program T;
         type UserId = distinct integer;
         function Add<T: Numeric>(Left: T; Right: T): T; begin return Left + Right; end function;
         function Show<T: Printable>(Value: T): T; begin return Value; end function;
         begin
           const A: UserId := Add(UserId(1), UserId(2));
           const B: UserId := Show(UserId(1));
         end.",
    );
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors
            .iter()
            .all(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
}
