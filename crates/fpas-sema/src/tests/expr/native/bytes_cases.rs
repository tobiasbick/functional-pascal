//! Named/positional byte-buffer fixtures independent of catalog generation.

pub(super) fn cases() -> Vec<(String, String)> {
    let buffer = "Result of (Bytes.ByteBuffer, Bytes.BufferError)";
    [
        ("Length", "integer", "", ""),
        ("IsEmpty", "boolean", "", ""),
        ("Capacity", "integer", "", ""),
        ("ToArray", "Result of (array of integer, Bytes.BufferError)", "", ""),
        ("Slice", buffer, "0, 1", "Len := 1, Start := 0"),
        ("Concat", buffer, "Receiver", "B := Receiver"),
        ("SetAt", buffer, "0, 9", "Value := 9, Index := 0"),
        ("Fill", buffer, "7, 0, 1", "Len := 1, Start := 0, Value := 7"),
        ("CopyFrom", buffer, "Receiver, 0, 1, 1", "Len := 1, TargetStart := 1, SourceStart := 0, Source := Receiver"),
        ("Resize", buffer, "4", "Length := 4"),
    ].into_iter().map(|(operation, result, positional, named)| {
        let source = |arguments| format!("program T; uses Std.Bytes as Bytes; begin var Receiver := Bytes.FromArray([1, 2, 3]).Unwrap(); const Output: {result} := Receiver.{operation}({arguments}); end.");
        (source(positional), source(named))
    }).collect()
}
