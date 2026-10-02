//! Semantic tests for generic methods on non-generic record types.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md` (Generic record methods)

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

// ---------------------------------------------------------------------------
// Positive (valid) cases
// ---------------------------------------------------------------------------

#[test]
fn generic_function_method_on_plain_record() {
    check_ok(
        r#"program T;
          type Box = record
           Value: integer;
           function Map<R>(Self: Box; F: function(X: integer): R): R;
           begin return F(Self.Value); end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_procedure_method_on_plain_record() {
    check_ok(
        r#"program T;
          type Wrapper = record
           Value: integer;
           procedure Apply<T>(Self: Wrapper; F: function(X: integer): T);
           begin var _ : T := F(Self.Value); end procedure;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_with_comparable_constraint() {
    check_ok(
        r#"program T;
          type Container = record
           Value: integer;
           function MaxWith<T: Comparable>(Self: Container; Other: T): T;
           begin if Self.Value > 0 then return Other;
             else return Other; end if;
           end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_with_numeric_constraint() {
    check_ok(
        r#"program T;
          type Accumulator = record
           Base: integer;
           function Add<T: Numeric>(Self: Accumulator; Extra: T): T;
           begin return Extra; end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_multiple_type_params() {
    check_ok(
        r#"program T;
          type Pair = record
           First: integer;
           Second: string;
           function Swap<A, B>(Self: Pair; X: A; Y: B): A;
           begin return X; end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_called_with_inferred_type() {
    check_ok(
        r#"program T;
          type Box = record
           Value: integer;
           function Map<R>(Self: Box; F: function(X: integer): R): R;
           begin return F(Self.Value); end function;
         end record;
         function Stringify(X: integer): string;
         begin return 'x'; end function;
          var B: Box := record Value := 42; end record;
          var S: string := B.Map(Stringify);
         begin null; end program;"#,
    );
}

#[test]
fn two_records_each_with_independent_generic_methods() {
    check_ok(
        r#"program T;
          type Box = record
           Value: integer;
           function Map<R>(Self: Box; F: function(X: integer): R): R;
           begin return F(Self.Value); end function;
         end record;
          type Cell = record
           Value: string;
           function Into<R>(Self: Cell; F: function(X: string): R): R;
           begin return F(Self.Value); end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_body_can_declare_local_of_generic_type_and_return_direct_call() {
    check_ok(
        r#"program T;
          type Holder = record
           Value: integer;
           function Wrap<R>(Self: Holder; F: function(X: integer): R): R;
           begin var Local: R := F(Self.Value);
             return F(Self.Value);
           end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_body_returning_local_generic_variable_reproducer() {
    check_ok(
        r#"program T;
          type Holder = record
           Value: integer;
           function Wrap<R>(Self: Holder; F: function(X: integer): R): R;
           begin var Local: R := F(Self.Value);
             return Local;
           end function;
         end record;
         begin null; end program;"#,
    );
}

// ---------------------------------------------------------------------------
// Negative (error) cases
// ---------------------------------------------------------------------------

#[test]
fn generic_method_constraint_violation_at_call_site() {
    let errors = check_errors(
        r#"program T;
          type Box = record
           Value: integer;
           function AddTwo<T: Numeric>(Self: Box; X: T): T;
           begin return X; end function;
         end record;
          var B: Box := record Value := 1; end record;
          var S: string := B.AddTwo('hello');
         begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_CONSTRAINT_VIOLATION),
        "expected constraint-violation diagnostic, got: {errors:#?}"
    );
}

#[test]
fn generic_method_missing_self_param_is_rejected() {
    let errors = check_errors(
        r#"program T;
          type Box = record
           Value: integer;
           function Bad<R>(X: integer): R;
           begin return X; end function;
         end record;
         begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected type-mismatch diagnostic for missing Self, got: {errors:#?}"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn generic_method_type_param_shadows_outer_name_is_ok() {
    // The method-level `T` is a new scope; it does not conflict with outer names.
    check_ok(
        r#"program T;
          type Container = record
           Value: integer;
           function Pick<T>(Self: Container; Other: T): T;
           begin return Other; end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_with_no_type_params_still_valid() {
    // Ensure non-generic methods on records still work correctly after the change.
    check_ok(
        r#"program T;
          type Counter = record
           Value: integer;
           function Incr(Self: Counter): integer;
           begin return Self.Value + 1; end function;
         end record;
         begin null; end program;"#,
    );
}

#[test]
fn generic_method_return_type_is_generic_param() {
    check_ok(
        r#"program T;
          type Identity = record
           function Id<T>(Self: Identity; X: T): T;
           begin return X; end function;
         end record;
         begin null; end program;"#,
    );
}
