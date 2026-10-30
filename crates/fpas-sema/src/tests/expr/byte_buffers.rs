//! Buffer capabilities and writable-storage rules apply uniformly to native values.
use super::{check_errors, check_ok};

#[test]
fn buffers_reject_implicit_conversions_ordering_and_alternate_mutation_paths() {
    for body in [
        "const B := Bytes.Create(2).Unwrap(); discard B.SetAtInPlace(0, 1);",
        "discard Bytes.Create(2).Unwrap().ResizeInPlace(3);",
        "var B := Bytes.Create(2).Unwrap(); const F := B.SetAtInPlace;",
        "var B := Bytes.Create(2).Unwrap(); B[0] := 7;",
        "const B := Bytes.Create(2).Unwrap(); const X := B['0'];",
        "const B: Bytes.ByteBuffer := [1, 2];",
        "const B := Bytes.Create(2).Unwrap(); const A: array of integer := B;",
        "const B := Bytes.Create(2).Unwrap(); const X := B < B;",
        "const B := Bytes.Create(2).Unwrap(); const X := [B].Sort();",
        "const B := Bytes.Create(2).Unwrap(); const N := Bytes.Length(B);",
        "const B := Bytes.Create(2).Unwrap(); const X := B.Fill(Byte := 1, Start := 0, Len := 1);",
        "var B := Bytes.Create(2).Unwrap(); for V: integer in B do discard B.ResizeInPlace(3); end for;",
    ] {
        check_errors(&format!(
            "program T; uses Std.Bytes as Bytes; begin {body} end."
        ));
    }
}

#[test]
fn capacity_mutation_supports_named_arguments_and_nested_writable_slots() {
    check_ok(
        r#"
program T;
uses Std.Bytes as Bytes;
type Packet = record Data: Bytes.ByteBuffer; end record;
procedure Grow(var B: Bytes.ByteBuffer);
begin
  discard B.ReserveInPlace(Capacity := 64);
  discard B.TrimCapacityInPlace();
end procedure;
begin
  var P := Packet(Data := Bytes.Create(2).Unwrap());
  Grow(var P.Data);
  var Values := [P.Data];
  discard Values[0].ResizeInPlace(Length := 4);
  discard Values[0].ReserveInPlace(100);
end.
"#,
    );
}

#[test]
fn indexed_bytes_cannot_escape_as_writable_integer_references() {
    check_errors(
        "program T; uses Std.Bytes as Bytes; procedure SetValue(var X: integer); begin X := 1; end procedure; begin var B := Bytes.Create(2).Unwrap(); SetValue(var B[0]); end.",
    );
}
