# `Std.Text.Utf8`

Strict native conversion between FPAS strings and `Std.Bytes.ByteBuffer`.
Import `Std.Text.Utf8 as Utf8` and `Std.Bytes as ByteStorage`.

| Kind | Name | Notes |
| --- | --- | --- |
| function | `Encode(Text: string): Result of (ByteStorage.ByteBuffer, ByteStorage.BufferError)` | Encode directly into initialized native byte storage; capacity equals encoded byte length |
| function | `Decode(Bytes: ByteStorage.ByteBuffer): Result of (string, Utf8.Utf8Error)` | Strictly validate and copy UTF-8 into one string |
| function | `AppendInPlace(var Buffer: ByteStorage.ByteBuffer; Text: string): Result of (boolean, ByteStorage.BufferError)` | Append text in one native bulk operation without an intermediate encoded buffer |

Buffer positions count bytes; string indexing counts Unicode scalars. Empty
values round-trip. Encoding and decoding preserve NUL, BOM, combining sequences
and all other contents. Decode never strips, repairs or normalizes text.

## Errors and offsets

Encode and AppendInPlace return `ByteStorage.BufferError.LimitExceeded(Limit)`
or `AllocationFailed(RequestedBytes)` for expected storage failures. The general
buffer limit is 268435456 bytes (256 MiB). AppendInPlace commits neither bytes
nor capacity on failure; success is `Ok(true)`, including appending empty text.
Use `try`, `case` or `.Unwrap()` to consume these Results.

Decode returns the source unit's `Utf8Error`:

| Variant | Meaning |
| --- | --- |
| `InvalidByte(Offset, Detail)` | Malformed, overlong, surrogate or out-of-range UTF-8 sequence |
| `IncompleteSequence(Offset)` | Input ends within an otherwise valid sequence |
| `AllocationFailed(RequestedBytes)` | Recoverable allocation of the resulting string failed |

Offset is the zero-based **start of the first invalid or incomplete sequence**,
as established by Rust's UTF-8 validator. It does not point at a later bad
continuation byte. Detail is explanatory text, not a machine-matched contract.
For `[120, 226, 40, 161]`, the invalid sequence starts at offset 1.

A buffer can hold malformed UTF-8 unchanged. Integer byte-domain validation
belongs to `ByteStorage.FromArray`, before text decoding or I/O. There is no
implicit array conversion or public incremental decoder. HTTP/SSE owns bounded
fragment buffering and finalization, including sequences split across chunks.

```pascal id=utf8-bytes run
program Utf8Bytes;
uses Std.Bytes as ByteStorage, Std.Text.Utf8 as Utf8, Std.Test as Test;
begin
  var Data := Utf8.Encode('Grüße'#0).Unwrap();
  const Before := Data;
  discard Utf8.AppendInPlace(var Data, '🌍').Unwrap();
  Test.AssertEquals('Grüße'#0, Utf8.Decode(Before).Unwrap());
  Test.AssertEquals('Grüße'#0'🌍', Utf8.Decode(Data).Unwrap());
  const Invalid := ByteStorage.FromArray([120, 226, 40, 161]).Unwrap();
  Test.AssertTrue(Utf8.Decode(Invalid) is Error(Utf8.Utf8Error.InvalidByte(1, _)));
end.
```

## Implementation (contributors)

| Concern | Location |
| --- | --- |
| Public declarations and atomic writable wrapper | [`Utf8.fpas`](../../../../lib/Std/Text/Utf8.fpas) |
| Native codec and allocation handling | [`str/utf8.rs`](../../../../crates/fpas-runtime/src/stdlib/str/utf8.rs) |
| Managed storage | [`value/bytes.rs`](../../../../crates/fpas-runtime/src/value/bytes.rs) |
| Boundary tests | [`utf8_boundaries_test.fpas`](../../../../tests/stdlib/net/utf8_boundaries_test.fpas) |

## See also

- [Managed bytes](../bytes.md)
- [Networking index](README.md)
- [`Std.Net.Sockets`](net.md)
