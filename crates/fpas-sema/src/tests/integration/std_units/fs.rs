//! Binary file signatures retain nominal filesystem errors and public argument labels.

use super::{check_errors, check_ok};

#[test]
fn binary_files_have_typed_results_and_named_parameters() {
    check_ok(
        "program Binary; uses Std.Fs as Fs; begin const Read: result of (array of integer, Fs.FsError) := Fs.ReadBytes(Path := 'input'); const Written: result of (boolean, Fs.FsError) := Fs.WriteBytes(Bytes := [0, 255], Path := 'output'); end.",
    );
}

#[test]
fn binary_files_reject_text_or_real_byte_arrays_and_wrong_argument_labels() {
    for call in [
        "Fs.ReadBytes(42)",
        "Fs.ReadBytes(Bytes := 'input')",
        "Fs.WriteBytes('output', 'text')",
        "Fs.WriteBytes('output', [1.0])",
        "Fs.WriteBytes(Path := 'output', Text := 'text')",
        "Fs.WriteBytes('output')",
    ] {
        assert!(
            !check_errors(&format!(
                "program Invalid; uses Std.Fs as Fs; begin discard {call}; end."
            ))
            .is_empty(),
            "accepted {call}"
        );
    }
}

#[test]
fn file_handles_expose_nominal_modes_and_typed_cancellable_operations() {
    check_ok(
        r#"program Handles;
uses Std.Fs as Fs, Std.Tasks as Tasks;
function OpenFile(): result of (Fs.File, Fs.FsError);
begin return Fs.Open(Path := 'data', Mode := Fs.FileMode.Read); end function;
procedure Probe(File: Fs.File; Token: Tasks.CancellationToken);
begin
  const Closed: result of (boolean, Fs.FsError) := Fs.Close(File := File);
  const Bytes: result of (array of integer, Fs.FsError) := Fs.Read(MaxBytes := 2, File := File);
  const Count: result of (integer, Fs.FsError) := Fs.Write(Bytes := [0, 255], File := File);
  const Line: result of (option of string, Fs.FsError) := Fs.ReadLine(File := File);
  const Offset: result of (integer, Fs.FsError) := Fs.Seek(Origin := Fs.SeekOrigin.Start, Offset := 0, File := File);
  const Position: result of (integer, Fs.FsError) := Fs.Tell(File);
  const Flushed: result of (boolean, Fs.FsError) := Fs.Flush(File);
  const Read: result of (array of integer, Fs.FsError) := Fs.ReadWithCancellation(Token := Token, MaxBytes := 2, File := File);
  const Written: result of (integer, Fs.FsError) := Fs.WriteWithCancellation(Token := Token, Bytes := [0], File := File);
  const ReadLine: result of (option of string, Fs.FsError) := Fs.ReadLineWithCancellation(Token := Token, File := File);
end procedure;
begin end."#,
    );
}

#[test]
fn file_handles_reject_wrong_modes_origins_payloads_and_tokens() {
    for call in [
        "Fs.Open('data', Fs.SeekOrigin.Start)",
        "Fs.Seek(File, 0, Fs.FileMode.Read)",
        "Fs.Read(File, '2')",
        "Fs.Write(File, 'text')",
        "Fs.ReadWithCancellation(File, 2, File)",
        "Fs.ReadLine(File, 2)",
    ] {
        assert!(!check_errors(&format!(
            "program Invalid; uses Std.Fs as Fs; procedure Probe(File: Fs.File); begin discard {call}; end procedure; begin end."
        )).is_empty(), "accepted {call}");
    }
}
