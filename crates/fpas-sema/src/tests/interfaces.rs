//! Semantic analysis tests for `interface` / `implements` / `extends`.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Interfaces)

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

// ---------------------------------------------------------------------------
// Positive (valid) cases
// ---------------------------------------------------------------------------

#[test]
fn interface_empty_is_valid() {
    check_ok(
        "program T;
         type INothing = interface
         end;
         begin end.",
    );
}

#[test]
fn interface_with_function_method() {
    check_ok(
        "program T;
         type IShape = interface
           function Area(Self: IShape): integer;
         end;
         begin end.",
    );
}

#[test]
fn interface_with_procedure_method() {
    check_ok(
        "program T;
         type ILogger = interface
           procedure Log(Self: ILogger);
         end;
         begin end.",
    );
}

#[test]
fn record_implements_simple_interface() {
    check_ok(
        "program T;
         type IGreeter = interface
           function Greet(Self: IGreeter): string;
         end;
         type Dog = record
           implements IGreeter;
           function Greet(Self: Dog): string;
           begin return 'Woof' end;
         end;
         begin end.",
    );
}

#[test]
fn record_implements_procedure_interface() {
    check_ok(
        "program T;
         type IRunner = interface
           procedure Run(Self: IRunner);
         end;
         type Person = record
           Name: string;
           implements IRunner;
           procedure Run(Self: Person);
           begin end;
         end;
         begin end.",
    );
}

#[test]
fn record_implements_multiple_interfaces() {
    check_ok(
        "program T;
         type IWalkable = interface
           procedure Walk(Self: IWalkable);
         end;
         type ITalkable = interface
           function Talk(Self: ITalkable): string;
         end;
         type Human = record
           implements IWalkable;
           implements ITalkable;
           procedure Walk(Self: Human);
           begin end;
           function Talk(Self: Human): string;
           begin return 'hello' end;
         end;
         begin end.",
    );
}

#[test]
fn interface_extends_parent() {
    check_ok(
        "program T;
         type IBase = interface
           function Id(Self: IBase): integer;
         end;
         type IDerived = interface
           extends IBase;
           function Name(Self: IDerived): string;
         end;
         begin end.",
    );
}

#[test]
fn record_implements_derived_interface() {
    check_ok(
        "program T;
         type IBase = interface
           function Id(Self: IBase): integer;
         end;
         type IDerived = interface
           extends IBase;
           function Name(Self: IDerived): string;
         end;
         type Thing = record
           implements IDerived;
           function Id(Self: Thing): integer;
           begin return 1 end;
           function Name(Self: Thing): string;
           begin return 'thing' end;
         end;
         begin end.",
    );
}

// ---------------------------------------------------------------------------
// Negative (error) cases
// ---------------------------------------------------------------------------

#[test]
fn record_missing_interface_method_errors() {
    let errors = check_errors(
        "program T;
         type IShape = interface
           function Area(Self: IShape): integer;
         end;
         type Square = record
           implements IShape;
           { Area is NOT defined here }
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for missing method, got: {errors:#?}"
    );
}

#[test]
fn record_wrong_method_kind_errors() {
    let errors = check_errors(
        "program T;
         type IShape = interface
           function Area(Self: IShape): integer;
         end;
         type Square = record
           implements IShape;
           { Area is a procedure, but interface requires a function }
           procedure Area(Self: Square);
           begin end;
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for method kind mismatch, got: {errors:#?}"
    );
}

#[test]
fn record_method_wrong_param_count_errors() {
    let errors = check_errors(
        "program T;
         type ICalc = interface
           function Add(Self: ICalc; A: integer; B: integer): integer;
         end;
         type Calc = record
           implements ICalc;
           { Missing B parameter }
           function Add(Self: Calc; A: integer): integer;
           begin return A end;
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for wrong param count, got: {errors:#?}"
    );
}

#[test]
fn record_method_wrong_param_type_errors() {
    let errors = check_errors(
        "program T;
         type ICalc = interface
           function Scale(Self: ICalc; Factor: real): real;
         end;
         type Calc = record
           implements ICalc;
           { Factor is integer, interface requires real }
           function Scale(Self: Calc; Factor: integer): real;
           begin return 1.0 end;
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for wrong param type, got: {errors:#?}"
    );
}

#[test]
fn record_method_wrong_return_type_errors() {
    let errors = check_errors(
        "program T;
         type IInfo = interface
           function Value(Self: IInfo): integer;
         end;
         type Info = record
           implements IInfo;
           { Returns real, interface requires integer }
           function Value(Self: Info): real;
           begin return 1.0 end;
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for wrong return type, got: {errors:#?}"
    );
}

#[test]
fn implements_non_interface_errors() {
    let errors = check_errors(
        "program T;
         type Point = record X: integer; end;
         type Bad = record
           implements Point; { Point is not an interface }
         end;
         begin end.",
    );
    assert!(!errors.is_empty(), "expected an error for non-interface in implements");
}

#[test]
fn implements_unknown_name_errors() {
    let errors = check_errors(
        "program T;
         type Bad = record
           implements IDoesNotExist;
         end;
         begin end.",
    );
    assert!(!errors.is_empty(), "expected an error for unknown interface name");
}
