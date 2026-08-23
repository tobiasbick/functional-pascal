# `Std.Net.Utf8`

Pure FPAS conversion between strings and byte arrays.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Encode(Text: string): array of integer` | produces UTF-8 bytes in `0..255` |
| function | `Decode(Bytes: array of integer): Result of string, string` | rejects malformed, overlong, surrogate, and out-of-range sequences |

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Utf8.fpas`](../../../../lib/Std/Net/Utf8.fpas) |
| Regression tests | [`utf8_roundtrip_test.fpas`](../../../../tests/stdlib/net/utf8_roundtrip_test.fpas) |

## See also

- [Networking index](README.md)
- [`Std.Net`](net.md)
