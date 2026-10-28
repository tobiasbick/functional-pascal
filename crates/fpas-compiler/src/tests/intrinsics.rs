use super::*;

mod bits;

#[test]
fn binary_files_keep_their_compiler_catalog_and_argument_shapes() {
    use fpas_runtime::{FsIntrinsic, Intrinsic, Opcode};

    assert_eq!(
        crate::intrinsic_catalog::resolve("Std.Fs.ReadBytes", None),
        Some(Intrinsic::Fs(FsIntrinsic::ReadBytes))
    );
    assert_eq!(
        crate::intrinsic_catalog::resolve("Std.Fs.WriteBytes", None),
        Some(Intrinsic::Fs(FsIntrinsic::WriteBytes))
    );
    let program = parse_ok(
        "program Binary; uses Std.Fs as Fs; begin discard Fs.ReadBytes(Path := 'input'); discard Fs.WriteBytes(Bytes := [0, 255], Path := 'output'); end.",
    );
    let executable = crate::compile(&program).expect("binary file intrinsic compilation");
    let calls = executable
        .executable()
        .code
        .iter()
        .filter(|instruction| instruction.opcode() == Ok(Opcode::Intrinsic))
        .filter_map(|instruction| {
            let operands = instruction.abc_operands().expect("intrinsic operands");
            match Intrinsic::from_u16(operands.b) {
                Some(Intrinsic::Fs(
                    operation @ (FsIntrinsic::ReadBytes | FsIntrinsic::WriteBytes),
                )) => Some((operation, operands.auxiliary)),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls,
        vec![(FsIntrinsic::ReadBytes, 1), (FsIntrinsic::WriteBytes, 2)]
    );
}

#[test]
fn binary_write_named_arguments_keep_written_evaluation_order() {
    assert_succeeds(
        r#"program BinaryOrder;
uses Std.Fs as Fs, Std.Test as Test;
var Order: string := '';
function Data(): array of integer;
begin
  Order := Order + 'B';
  return [256];
end function;
function Target(): string;
begin
  Order := Order + 'P';
  return 'invalid-bytes-must-not-open-this-file';
end function;
begin
  Test.AssertTrue(Fs.WriteBytes(Bytes := Data(), Path := Target()) is Error(Fs.FsError.InvalidByte(0, 256)));
  Test.AssertEquals('BP', Order);
end."#,
    );
}

#[test]
fn binary_text_codecs_lower_to_standard_intrinsics_with_nominal_errors() {
    assert_succeeds(
        r#"program Codecs;
uses Std.Encoding as Codec, Std.Test as Test;

function DecodeAndHex(Text: string): result of (string, Codec.EncodingError);
begin
  const Bytes := try Codec.Base64Decode(Text);
  return Codec.HexEncode(Bytes);
end function;

begin
  Test.AssertTrue(Codec.Base64Encode([102]) is Ok('Zg=='));
  const Standard := Codec.Base64Decode('Zg==').Unwrap();
  Test.AssertEquals(1, Standard.Length());
  Test.AssertEquals(102, Standard[0]);
  Test.AssertTrue(Codec.Base64UrlEncode([255]) is Ok('_w'));
  const Url := Codec.Base64UrlDecode('_w').Unwrap();
  Test.AssertEquals(1, Url.Length());
  Test.AssertEquals(255, Url[0]);
  Test.AssertTrue(Codec.HexEncode([255]) is Ok('FF'));
  const Hex := Codec.HexDecode('Ff').Unwrap();
  Test.AssertEquals(1, Hex.Length());
  Test.AssertEquals(255, Hex[0]);
  Test.AssertTrue(Codec.HexEncode([256]) is Error(Codec.EncodingError.InvalidByte(0, 256)));
  Test.AssertTrue(DecodeAndHex('Zg==') is Ok('66'));
  Test.AssertTrue(DecodeAndHex('Zh==') is Error(Codec.EncodingError.NonCanonical(1)));
  const ExplicitError := Codec.EncodingError.InvalidByte(7, 256);
  Test.AssertTrue(ExplicitError.Message().StartsWith('Std.Encoding.EncodingError.InvalidByte: '));
end."#,
    );
}

#[test]
fn lexical_path_helpers_lower_through_the_standard_intrinsic_catalog() {
    assert_succeeds(
        "program PathHelpers; uses Std.Path as Path, Std.Test as Test; begin const Absolute: boolean := Path.IsAbsolute('file'); Test.AssertFalse(Absolute); const Relative: option of string := Path.RelativeTo(Target := 'a/file', Base := 'a'); Test.AssertTrue(Relative is Some('file')); const Changed: option of string := Path.ChangeExtension(Path := 'file.txt', Extension := 'log'); Test.AssertTrue(Changed is Some('file.log')); end.",
    );
}

#[test]
fn interface_backed_program_keeps_short_standard_intrinsic_dispatch() {
    let program = parse_ok(
        "program RegisterInterfaceIntrinsic;\nuses Std.Console as Console;\nbegin\n  Console.WriteLn('hello');\nend.",
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
        "program RuntimeLayouts;\nuses Std.Json as Json;\nbegin\n  const Parsed: result of (Json.JsonValue, Json.JsonError) := Json.Parse('null');\nend.",
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
  const Origin: Point := Point(
    X := 1
  );
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
        "program RegisterIntrinsics;\nuses Std.Math as Math, Std.Conv as Conv, Std.Test as Test;\nbegin\n  const Text: string := 'fpas'.ToUpper();\n  const Root: real := Math.Sqrt(81.0);\n  const Number: string := Conv.IntToStr(42);\n  const Formatted: string := 'n=%d %s'.Format(42, 'ok');\n  Test.AssertEquals('FPAS', Text);\n  Test.AssertEquals(9.0, Root);\n  Test.AssertEquals('42', Number);\n  Test.AssertEquals('n=42 ok', Formatted);\nend.",
    );
    assert_eq!(
        execution.value.try_to_data().expect("pure return value"),
        fpas_runtime::Value::Unit
    );
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
        .find(|instruction| instruction.opcode() == Ok(fpas_runtime::Opcode::Intrinsic))
        .expect("intrinsic opcode");
    let operands = instruction.abc_operands().expect("ABC operands");
    assert_ne!(operands.a, fpas_runtime::NO_REGISTER);
    assert_eq!(
        fpas_runtime::Intrinsic::from_u16(operands.b),
        Some(fpas_runtime::Intrinsic::Str(
            fpas_runtime::StrIntrinsic::Length,
        ))
    );
    assert_eq!(operands.auxiliary, 1);
}

#[test]
fn higher_order_intrinsics_invoke_numeric_callbacks() {
    let execution = assert_succeeds(
        "program RegisterCallbacks;\nuses Std.Test as Test;\n\nfunction Double(Value: integer): integer;\nbegin\n  return Value * 2;\nend function;\n\nbegin\n  const Values: array of integer := [2, 3, 4].Map(Double);\n  Test.AssertEquals(3, Values.Length());\n  Test.AssertEquals(6, Values[1]);\nend.",
    );
    assert_eq!(
        execution.value.try_to_data().expect("pure return value"),
        fpas_runtime::Value::Unit
    );
}

#[test]
fn intrinsic_temporaries_do_not_clobber_loop_state() {
    assert_succeeds(
        "program RegisterIntrinsicLoop;\nuses Std.Test as Test;\nbegin\n  var Total: integer := 0;\n  for Index: integer := 1 to 3 do\n  begin\n    Total := Total + 'abc'.Length();\n  end; end for;\n  Test.AssertEquals(9, Total);\nend.",
    );
}

#[test]
fn variadic_console_output_preserves_evaluation_order() {
    let execution = assert_succeeds(
        "program RegisterConsoleOutput;\nuses Std.Console as Console, Std.Test as Test;\n\nfunction SideEffect(): string;\nbegin\n  Console.WriteText('B');\n  return 'C';\nend function;\n\nbegin\n  Console.WriteText('A', SideEffect());\n  Console.WriteLn('D', 42, true);\n  Console.WriteLn();\n  Test.AssertScreenLine('ABCD42true', 1);\n  Test.AssertScreenLine('', 2);\nend.",
    );
    assert_eq!(
        execution.value.try_to_data().expect("pure return value"),
        fpas_runtime::Value::Unit
    );
}

#[test]
fn hosted_file_calls_compile_all_public_argument_shapes() {
    use fpas_runtime::{FsIntrinsic, Intrinsic};
    let operations = [
        ("Open", FsIntrinsic::Open),
        ("Close", FsIntrinsic::Close),
        ("Read", FsIntrinsic::Read),
        ("Write", FsIntrinsic::Write),
        ("ReadLine", FsIntrinsic::ReadLine),
        ("Seek", FsIntrinsic::Seek),
        ("Tell", FsIntrinsic::Tell),
        ("Flush", FsIntrinsic::Flush),
        ("ReadWithCancellation", FsIntrinsic::ReadWithCancellation),
        ("WriteWithCancellation", FsIntrinsic::WriteWithCancellation),
        (
            "ReadLineWithCancellation",
            FsIntrinsic::ReadLineWithCancellation,
        ),
    ];
    for (name, operation) in operations {
        assert_eq!(
            crate::intrinsic_catalog::resolve(&format!("Std.Fs.{name}"), None),
            Some(Intrinsic::Fs(operation))
        );
    }
    let program = parse_ok(
        r#"program Handles;
uses Std.Fs as Fs, Std.Tasks as Tasks;
function OpenFile(): result of (Fs.File, Fs.FsError);
begin return Fs.Open('data', Fs.FileMode.Read); end function;
procedure Probe(File: Fs.File; Token: Tasks.CancellationToken);
begin
  discard Fs.Close(File);
  discard Fs.Read(MaxBytes := 2, File := File);
  discard Fs.Write(Bytes := [0, 255], File := File);
  discard Fs.ReadLine(File);
  discard Fs.Seek(Origin := Fs.SeekOrigin.Start, Offset := 0, File := File);
  discard Fs.Tell(File);
  discard Fs.Flush(File);
  discard Fs.ReadWithCancellation(Token := Token, MaxBytes := 2, File := File);
  discard Fs.WriteWithCancellation(Token := Token, Bytes := [0], File := File);
  discard Fs.ReadLineWithCancellation(Token := Token, File := File);
end procedure;
begin end."#,
    );
    crate::compile(&program).expect("all hosted file call signatures compile");
}

#[test]
fn default_process_options_use_the_pure_runtime_record_factory() {
    assert_succeeds(
        r#"program Defaults;
uses Std.Proc as Proc, Std.Test as Test;
begin
  const Defaults: Proc.ProcessOptions := Proc.DefaultOptions();
  Test.AssertTrue(Defaults.WorkingDirectory is None);
  Test.AssertEquals(0, Defaults.Environment.Length());
  Test.AssertEquals('', Defaults.Input);
  Test.AssertTrue(Defaults.TimeoutMillis is None);
  Test.AssertEquals(1048576, Defaults.MaxOutputBytes);
  Test.AssertTrue(Defaults.CancellationToken is None);
  const Changed := Defaults with Input := 'new'; end with;
  Test.AssertEquals('new', Changed.Input);
  Test.AssertEquals('', Defaults.Input);
end."#,
    );
    for (name, operation) in [
        (
            "RunWithOptions",
            fpas_runtime::ProcIntrinsic::RunWithOptions,
        ),
        (
            "RunCaptureWithOptions",
            fpas_runtime::ProcIntrinsic::RunCaptureWithOptions,
        ),
        (
            "DefaultOptions",
            fpas_runtime::ProcIntrinsic::DefaultOptions,
        ),
    ] {
        assert_eq!(
            crate::intrinsic_catalog::resolve(&format!("Std.Proc.{name}"), None),
            Some(fpas_runtime::Intrinsic::Proc(operation))
        );
    }
}
