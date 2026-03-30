//! Compiler integration tests for `interface` / `implements` virtual dispatch.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Interfaces)

use super::*;

// ---------------------------------------------------------------------------
// Basic dispatch
// ---------------------------------------------------------------------------

#[test]
fn interface_method_dispatch_via_concrete_var() {
    // A function accepting IAnimal calls the virtual method.
    let out = compile_and_run(
        "\
program Animals;
type IAnimal = interface
  function Speak(Self: IAnimal): string;
end;
type Dog = record
  implements IAnimal;
  function Speak(Self: Dog): string;
  begin return 'Woof' end;
end;
type Cat = record
  implements IAnimal;
  function Speak(Self: Cat): string;
  begin return 'Meow' end;
end;
begin
  var D: Dog := record end;
  var C: Cat := record end;
  Std.Console.WriteLn(D.Speak());
  Std.Console.WriteLn(C.Speak())
end.",
    );
    assert_eq!(out.lines, vec!["Woof", "Meow"]);
}

#[test]
fn interface_function_returns_correct_value_direct_type() {
    let out = compile_and_run(
        "\
program DirectIface;
type IValue = interface
  function Get(Self: IValue): integer;
end;
type Box = record
  Val: integer;
  implements IValue;
  function Get(Self: Box): integer;
  begin return Self.Val end;
end;
begin
  var B: Box := record Val := 42; end;
  Std.Console.WriteLn(B.Get())
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn interface_procedure_method_via_concrete_type() {
    let out = compile_and_run(
        "\
program IfaceProc;
type ILogger = interface
  procedure Log(Self: ILogger);
end;
type ConsoleLogger = record
  implements ILogger;
  procedure Log(Self: ConsoleLogger);
  begin
    Std.Console.WriteLn('logged')
  end;
end;
begin
  var L: ConsoleLogger := record end;
  L.Log()
end.",
    );
    assert_eq!(out.lines, vec!["logged"]);
}

#[test]
fn interface_method_with_extra_args() {
    let out = compile_and_run(
        "\
program IfaceArgs;
type ICalc = interface
  function Add(Self: ICalc; A: integer; B: integer): integer;
end;
type Calc = record
  implements ICalc;
  function Add(Self: Calc; A: integer; B: integer): integer;
  begin return A + B end;
end;
begin
  var C: Calc := record end;
  Std.Console.WriteLn(C.Add(3, 4))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

// ---------------------------------------------------------------------------
// Extends (interface inheritance)
// ---------------------------------------------------------------------------

#[test]
fn interface_extends_method_inherited() {
    let out = compile_and_run(
        "\
program IfaceExtends;
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
  begin return 7 end;
  function Name(Self: Thing): string;
  begin return 'thing' end;
end;
begin
  var T: Thing := record end;
  Std.Console.WriteLn(T.Id());
  Std.Console.WriteLn(T.Name())
end.",
    );
    assert_eq!(out.lines, vec!["7", "thing"]);
}

// ---------------------------------------------------------------------------
// Multiple interfaces
// ---------------------------------------------------------------------------

#[test]
fn record_implements_two_interfaces() {
    let out = compile_and_run(
        "\
program MultiIface;
type IWalkable = interface
  function Steps(Self: IWalkable): integer;
end;
type ITalkable = interface
  function Words(Self: ITalkable): integer;
end;
type Person = record
  implements IWalkable;
  implements ITalkable;
  function Steps(Self: Person): integer;
  begin return 100 end;
  function Words(Self: Person): integer;
  begin return 50 end;
end;
begin
  var P: Person := record end;
  Std.Console.WriteLn(P.Steps());
  Std.Console.WriteLn(P.Words())
end.",
    );
    assert_eq!(out.lines, vec!["100", "50"]);
}

// ---------------------------------------------------------------------------
// Sema error cases (compile-time failures)
// ---------------------------------------------------------------------------

#[test]
fn interface_missing_method_is_compile_error() {
    let (prog, parse_errs) = fpas_parser::parse(
        "\
program T;
type IShape = interface
  function Area(Self: IShape): integer;
end;
type Square = record
  implements IShape;
end;
begin end.",
    );
    assert!(parse_errs.is_empty());
    let err = crate::compile(&prog);
    assert!(
        err.is_err(),
        "compilation should fail when method is missing"
    );
}

#[test]
fn interface_method_kind_mismatch_is_compile_error() {
    let (prog, parse_errs) = fpas_parser::parse(
        "\
program T;
type IShape = interface
  function Area(Self: IShape): integer;
end;
type Square = record
  implements IShape;
  procedure Area(Self: Square);
  begin end;
end;
begin end.",
    );
    assert!(parse_errs.is_empty());
    assert!(crate::compile(&prog).is_err());
}

#[test]
fn implements_non_interface_is_compile_error() {
    let (prog, parse_errs) = fpas_parser::parse(
        "\
program T;
type Point = record X: integer; end;
type Bad = record
  implements Point;
end;
begin end.",
    );
    assert!(parse_errs.is_empty());
    assert!(crate::compile(&prog).is_err());
}
