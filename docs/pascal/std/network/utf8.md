# `Std.Text.Utf8`

Conversion between strings and UTF-8 byte arrays. Encoding uses one runtime pass over
the string bytes; decoding validates scalars and joins the resulting text once.

## Errors and offsets

`Decode` returns `Utf8Error.InvalidByte(Offset, Detail)` for byte-domain or malformed
sequence failures and `IncompleteSequence(Offset)` at its existing truncated
sequence check. Offsets remain zero-based byte offsets into the supplied array.
Encode remains infallible. There is no incremental Finish operation in this unit.
HTTP's SSE decoder owns buffering across chunks and wraps Decode failures as
`HttpError.Encoding(Cause)`, including a truncated final line at Sse.Finish.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Encode(Text: string): array of integer` | produces UTF-8 bytes in `0..255` |
| function | `Decode(Bytes: array of integer): Result of (string, Utf8Error)` | rejects malformed, overlong, surrogate, and out-of-range sequences |

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Utf8.fpas`](../../../../lib/Std/Text/Utf8.fpas) |
| Bulk encoder | [`str/mod.rs`](../../../../crates/fpas-runtime/src/stdlib/str/mod.rs), internal `EncodeBytes` intrinsic |
| Boundary tests | [`utf8_boundaries_test.fpas`](../../../../tests/stdlib/net/utf8_boundaries_test.fpas) |
| Regression tests | [`utf8_roundtrip_test.fpas`](../../../../tests/stdlib/net/utf8_roundtrip_test.fpas) |

## See also

- [Networking index](README.md)
- [`Std.Net.Sockets`](net.md)
