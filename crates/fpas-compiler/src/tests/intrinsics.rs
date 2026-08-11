use super::*;

#[test]
fn interface_backed_program_keeps_short_standard_intrinsic_dispatch() {
    let program = parse_ok(
        "\
program RegisterInterfaceIntrinsic;
uses Std.Console;
begin
  WriteLn('hello')
end.",
    );
    assert!(
        !fpas_sema::analyze_with_types(&program)
            .intrinsic_calls
            .is_empty(),
        "ordinary semantic analysis lost standard intrinsic metadata"
    );
    let metadata = fpas_sema::analyze_program_with_interface_support(&program, &[], &[])
        .expect("interface-backed semantic analysis");
    assert!(
        !metadata.intrinsic_calls.is_empty(),
        "interface-backed semantic analysis lost standard intrinsic metadata"
    );
    crate::compile_program_object_with_support(&program, &[], &[])
        .expect("interface-backed compilation should retain standard intrinsic metadata");
}

#[test]
fn object_retains_layouts_constructed_by_runtime_intrinsics() {
    let program = parse_ok(
        "\
program RuntimeLayouts;
uses Std.Graph, Std.Json;
begin
  var App: Application := Application.OpenForTest(4, 3);
  var Size: Std.Graph.Size := Application.Size(App);
  var Parsed: result of JsonValue, string := Parse('null');
  Application.Close(App)
end.",
    );
    let object = crate::compile_program_object_with_support(&program, &[], &[])
        .expect("runtime aggregate layouts must compile");

    assert!(
        object
            .records
            .iter()
            .any(|layout| layout.name.eq_ignore_ascii_case("Std.Graph.Application"))
    );
    assert!(
        object
            .records
            .iter()
            .any(|layout| layout.name.eq_ignore_ascii_case("Std.Graph.Size"))
    );
    assert!(
        object
            .enums
            .iter()
            .any(|layout| layout.name.eq_ignore_ascii_case("Std.Json.JsonValue"))
    );
}

#[test]
fn object_retains_layouts_referenced_by_portable_debug_types() {
    let program = parse_ok(
        r#"
program DebugLayout;

type
  Point = record
    X: integer;
  end;

begin
  var Origin: Point := record
    X := 1;
  end;
  var Marker: integer := Origin.X
end.
"#,
    );
    let object = crate::compile_program_object_with_support(&program, &[], &[])
        .expect("debug type layouts must survive object pruning");

    assert!(
        object
            .records
            .iter()
            .any(|layout| layout.name.eq_ignore_ascii_case("Point"))
    );
    assert!(object.debug_types.iter().any(|ty| matches!(
        ty,
        fpas_unit::object::ObjectDebugType::Record(name) if name.eq_ignore_ascii_case("Point")
    )));
}

#[test]
fn borrowed_standard_intrinsics_execute() {
    let execution = assert_succeeds(
        "\
program RegisterIntrinsics;
uses Std.Str, Std.Math, Std.Conv, Std.Test;
begin
  var Text: string := Std.Str.ToUpper('fpas');
  var Root: real := Std.Math.Sqrt(81.0);
  var Number: string := Std.Conv.IntToStr(42);
  var Formatted: string := Std.Str.Format('n=%d %s', 42, 'ok');
  Std.Test.AssertEquals('FPAS', Text);
  Std.Test.AssertEquals(9.0, Root);
  Std.Test.AssertEquals('42', Number);
  Std.Test.AssertEquals('n=42 ok', Formatted)
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_selection_uses_one_verified_register_window_convention() {
    let program = parse_ok(
        "\
program RegisterIntrinsicShape;
uses Std.Str;
begin
  if Std.Str.Length('abc') <> 3 then panic('bad')
end.",
    );
    let metadata = fpas_sema::analyze_with_types(&program);
    assert!(
        metadata.errors.is_empty(),
        "sema errors: {:?}",
        metadata.errors
    );
    assert!(
        !metadata.intrinsic_calls.is_empty(),
        "sema did not record intrinsic calls"
    );
    let executable =
        crate::compile(&program).expect("register intrinsic compilation should succeed");
    let instruction = executable
        .executable()
        .code
        .iter()
        .find(|instruction| instruction.opcode() == Ok(fpas_bytecode::Opcode::Intrinsic))
        .expect("intrinsic opcode");
    let operands = instruction.abc_operands().expect("ABC operands");
    assert_ne!(operands.a, fpas_bytecode::NO_REGISTER);
    assert_eq!(
        fpas_bytecode::Intrinsic::from_u16(operands.b),
        Some(fpas_bytecode::Intrinsic::Str(
            fpas_bytecode::StrIntrinsic::Length,
        ))
    );
    assert_eq!(operands.auxiliary, 1);
}

#[test]
fn higher_order_intrinsics_invoke_numeric_callbacks() {
    let execution = assert_succeeds(
        "\
program RegisterCallbacks;
uses Std.Array, Std.Test;

function Double(Value: integer): integer;
begin
  return Value * 2
end;

begin
  var Values: array of integer := Std.Array.Map([2, 3, 4], Double);
  Std.Test.AssertEquals(3, Std.Array.Length(Values));
  Std.Test.AssertEquals(6, Values[1])
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_temporaries_do_not_clobber_loop_state() {
    assert_succeeds(
        "\
program RegisterIntrinsicLoop;
uses Std.Str, Std.Test;
begin
  mutable var Total: integer := 0;
  for Index: integer := 1 to 3 do
  begin
    Total := Total + Std.Str.Length('abc')
  end;
  Std.Test.AssertEquals(9, Total)
end.",
    );
}

#[test]
fn variadic_console_output_preserves_evaluation_order() {
    let execution = assert_succeeds(
        "\
program RegisterConsoleOutput;
uses Std.Console, Std.Test;

function SideEffect(): string;
begin
  Std.Console.Write('B');
  return 'C'
end;

begin
  Std.Console.Write('A', SideEffect());
  Std.Console.WriteLn('D', 42, true);
  Std.Console.WriteLn();
  Std.Test.AssertScreenLine('ABCD42true', 1);
  Std.Test.AssertScreenLine('', 2)
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}
