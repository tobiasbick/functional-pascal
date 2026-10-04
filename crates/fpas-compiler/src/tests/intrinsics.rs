use super::*;

#[test]
fn interface_backed_program_keeps_short_standard_intrinsic_dispatch() {
    let program = parse_ok(
        r#"program RegisterInterfaceIntrinsic;
uses Std.Console as Console;
begin
  Console.WriteLn('hello');
end program;"#,
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
        r#"program RuntimeLayouts;
uses Std.Json as Json;
begin
  var Parsed: result of (Json.JsonValue, string) := Json.Parse('null');
end program;"#,
    );
    let object = crate::compile_program_object_with_support(&program, &[], &[])
        .expect("runtime aggregate layouts must compile");

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
        r#"program DebugLayout;

type Point = record
  X: integer;
end record;

begin
  var Origin: Point := Point(X := 1);
  var Marker: integer := Origin.X;
end program;
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
        r#"program RegisterIntrinsics;
uses Std.Str as Str; uses Std.Math as Math; uses Std.Conv as Conv; uses Std.Test as Test;
begin
  var Text: string := Str.ToUpper('fpas');
  var Root: real := Math.Sqrt(81.0);
  var Number: string := Conv.IntToStr(42);
  var Formatted: string := Str.Format('n=%d %s', 42, 'ok');
  Test.AssertEquals('FPAS', Text);
  Test.AssertEquals(9.0, Root);
  Test.AssertEquals('42', Number);
  Test.AssertEquals('n=42 ok', Formatted);
end program;"#,
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_selection_uses_one_verified_register_window_convention() {
    let program = parse_ok(
        r#"program RegisterIntrinsicShape;
uses Std.Str as Str;
begin
  if Str.Length('abc') <> 3 then panic('bad'); end if;
end program;"#,
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
        r#"program RegisterCallbacks;
uses Std.Arrays as Arrays; uses Std.Test as Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
  var Values: array of (integer) := Arrays.Map([2, 3, 4], Double);
  Test.AssertEquals(3, Arrays.Length(Values));
  Test.AssertEquals(6, Values[1]);
end program;"#,
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_temporaries_do_not_clobber_loop_state() {
    assert_succeeds(
        r#"program RegisterIntrinsicLoop;
uses Std.Str as Str; uses Std.Test as Test;
begin
  mutable var Total: integer := 0;
  for Index: integer := 1 to 3 do
  begin
    Total := Total + Str.Length('abc');
  end; end for;
  Test.AssertEquals(9, Total);
end program;"#,
    );
}

#[test]
fn variadic_console_output_preserves_evaluation_order() {
    let execution = assert_succeeds(
        r#"program RegisterConsoleOutput;
uses Std.Console as Console; uses Std.Test as Test;

function SideEffect(): string;
begin
  Console.WriteText('B');
  return 'C';
end function;

begin
  Console.WriteText('A', SideEffect());
  Console.WriteLn('D', 42, true);
  Console.WriteLn();
  Test.AssertScreenLine('ABCD42true', 1);
  Test.AssertScreenLine('', 2);
end program;"#,
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}
