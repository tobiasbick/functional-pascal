//! Generic functions accept explicit record parameters.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

// ---------------------------------------------------------------------------
// Positive (valid) cases
// ---------------------------------------------------------------------------

#[test]
fn generic_function_accepts_plain_record() {
    check_ok(
        r#"program T;
          type Box = record
           Value: integer;

         end record;

         function BoxMap of (R)(Receiver: Box; F: function(X: integer): R): R;
         begin return F(Receiver.Value); end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_procedure_accepts_plain_record() {
    check_ok(
        r#"program T;
          type Wrapper = record
           Value: integer;

         end record;

         procedure WrapperApply of (T)(Receiver: Wrapper; F: function(X: integer): T);
         begin const _ : T := F(Receiver.Value); end procedure;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_with_comparable_constraint() {
    check_ok(
        r#"program T;
          type Container = record
           Value: integer;

         end record;

         function ContainerMaxWith of (T: Comparable)(Receiver: Container; Other: T): T;
         begin if Receiver.Value > 0 then return Other;
           else return Other; end if;
         end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_with_numeric_constraint() {
    check_ok(
        r#"program T;
          type Accumulator = record
           Base: integer;

         end record;

         function AccumulatorAdd of (T: Numeric)(Receiver: Accumulator; Extra: T): T;
         begin return Extra; end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_multiple_type_params() {
    check_ok(
        r#"program T;
          type Pair = record
           First: integer;
           Second: string;

         end record;

         function PairSwap of (A, B)(Receiver: Pair; X: A; Y: B): A;
         begin return X; end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_called_with_inferred_type() {
    check_ok(
        r#"program T;

type Box = record
  Value: integer;


end record;

function BoxMap of (R)(Receiver: Box; F: function(X: integer): R): R;
begin
  return F(Receiver.Value);
end function;

function Stringify(X: integer): string;
begin
  return 'x';
end function;

const B: Box := Box(Value := 42);
const S: string := BoxMap(B, Stringify);

begin
  null;
end program;
"#,
    );
}

#[test]
fn two_records_each_with_independent_generic_functions() {
    check_ok(
        r#"program T;
          type Box = record
           Value: integer;

         end record;

         function BoxMap of (R)(Receiver: Box; F: function(X: integer): R): R;
         begin return F(Receiver.Value); end function;
          type Cell = record
           Value: string;

         end record;

         function CellInto of (R)(Receiver: Cell; F: function(X: string): R): R;
         begin return F(Receiver.Value); end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_body_can_declare_local_of_generic_type_and_return_direct_call() {
    check_ok(
        r#"program T;
          type Holder = record
           Value: integer;

         end record;

         function HolderWrap of (R)(Receiver: Holder; F: function(X: integer): R): R;
         begin const Local: R := F(Receiver.Value);
           return F(Receiver.Value);
         end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_body_returning_local_generic_variable_reproducer() {
    check_ok(
        r#"program T;
          type Holder = record
           Value: integer;

         end record;

         function HolderWrap of (R)(Receiver: Holder; F: function(X: integer): R): R;
         begin const Local: R := F(Receiver.Value);
           return Local;
         end function;
         begin null; end program;"#,
    );
}

// ---------------------------------------------------------------------------
// Negative (error) cases
// ---------------------------------------------------------------------------

#[test]
fn generic_record_function_constraint_violation_at_call_site() {
    let errors = check_errors(
        r#"program T;

type Box = record
  Value: integer;


end record;

function BoxAddTwo of (T: Numeric)(Receiver: Box; X: T): T;
begin
  return X;
end function;

const B: Box := Box(Value := 1);
const S: string := BoxAddTwo(B, 'hello');

begin
  null;
end program;
"#,
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_CONSTRAINT_VIOLATION),
        "expected constraint-violation diagnostic, got: {errors:#?}"
    );
}

#[test]
fn generic_return_rejects_unrelated_concrete_value() {
    let errors = check_errors(
        r#"program T;
          type Box = record
           Value: integer;
         end record;
         function Bad of (R)(X: integer): R;
         begin return X; end function;
         begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected type-mismatch diagnostic for unrelated generic result, got: {errors:#?}"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn generic_record_function_type_param_shadows_outer_name_is_ok() {
    // The routine-level `T` is a new scope; it does not conflict with outer names.
    check_ok(
        r#"program T;
          type Container = record
           Value: integer;

         end record;

         function ContainerPick of (T)(Receiver: Container; Other: T): T;
         begin return Other; end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_with_no_type_params_still_valid() {
    // Ordinary record parameters also work without generic type parameters.
    check_ok(
        r#"program T;
          type Counter = record
           Value: integer;

         end record;

         function CounterIncr(Receiver: Counter): integer;
         begin return Receiver.Value + 1; end function;
         begin null; end program;"#,
    );
}

#[test]
fn generic_record_function_return_type_is_generic_param() {
    check_ok(
        r#"program T;
          type Identity = record

         end record;

         function IdentityId of (T)(Receiver: Identity; X: T): T;
         begin return X; end function;
         begin null; end program;"#,
    );
}
