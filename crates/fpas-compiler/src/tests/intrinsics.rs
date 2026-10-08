use super::*;

mod bits;

#[test]
fn interface_backed_program_keeps_short_standard_intrinsic_dispatch() {
    let program = parse_ok(
        "\
program RegisterInterfaceIntrinsic;
uses Std.Console;
begin
  WriteLn('hello');
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
uses Std.Json;
begin
  const Parsed: result of JsonValue, string := Parse('null');
end.",
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
        r#"
program DebugLayout;

type
  Point = record
    X: integer;
  end record;

begin
  const Origin: Point := record
    X := 1;
  end;
  const Marker: integer := Origin.X;
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
        "program RegisterIntrinsics;\nuses Std.Math, Std.Conv, Std.Test;\nbegin\n  const Text: string := 'fpas'.ToUpper();\n  const Root: real := Std.Math.Sqrt(81.0);\n  const Number: string := Std.Conv.IntToStr(42);\n  const Formatted: string := 'n=%d %s'.Format(42, 'ok');\n  Std.Test.AssertEquals('FPAS', Text);\n  Std.Test.AssertEquals(9.0, Root);\n  Std.Test.AssertEquals('42', Number);\n  Std.Test.AssertEquals('n=42 ok', Formatted);\nend.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_selection_uses_one_verified_register_window_convention() {
    let program = parse_ok(
        "program RegisterIntrinsicShape;\n\nbegin\n  if 'abc'.Length() <> 3 then panic('bad'); end if;\nend.",
    );
    let metadata = fpas_sema::analyze_with_types(&program);
    assert!(
        metadata.errors.is_empty(),
        "sema errors: {:?}",
        metadata.errors
    );
    assert!(
        metadata
            .fluent_calls
            .values()
            .any(|call| call.name == "Std.Str.Length"),
        "sema did not record the type operation"
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
        "program RegisterCallbacks;\nuses Std.Test;\n\nfunction Double(Value: integer): integer;\nbegin\n  return Value * 2;\nend function;\n\nbegin\n  const Values: array of integer := [2, 3, 4].Map(Double);\n  Std.Test.AssertEquals(3, Values.Length());\n  Std.Test.AssertEquals(6, Values[1]);\nend.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn intrinsic_temporaries_do_not_clobber_loop_state() {
    assert_succeeds(
        "program RegisterIntrinsicLoop;\nuses Std.Test;\nbegin\n  var Total: integer := 0;\n  for Index: integer := 1 to 3 do\n  begin\n    Total := Total + 'abc'.Length();\n  end; end for;\n  Std.Test.AssertEquals(9, Total);\nend.",
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
  Std.Console.WriteText('B');
  return 'C';
end function;

begin
  Std.Console.WriteText('A', SideEffect());
  Std.Console.WriteLn('D', 42, true);
  Std.Console.WriteLn();
  Std.Test.AssertScreenLine('ABCD42true', 1);
  Std.Test.AssertScreenLine('', 2);
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}
