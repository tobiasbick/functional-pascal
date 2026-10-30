# `Std.Encoding`

Strict binary-to-text codecs for standard Base64, URL-safe Base64, and hexadecimal.
All six functions return a `Result` with the nominal `Encoding.EncodingError` family.
Binary inputs and outputs use `Std.Bytes.ByteBuffer`. Import `Std.Bytes as
ByteStorage` for buffer construction and type annotations. Reading a byte returns
an ordinary integer in `0..255`.

## Importing and names

Import `uses Std.Encoding as Encoding;` and call public functions as
`Encoding.Name`. Error variants are qualified as
`Encoding.EncodingError.Variant`. No filesystem, network, locale, or host
character encoding is involved.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Base64Encode(Bytes: ByteStorage.ByteBuffer): Result of (string, Encoding.EncodingError)` | encodes with the standard RFC 4648 alphabet and required `=` padding |
| function | `Base64Decode(Text: string): Result of (ByteStorage.ByteBuffer, Encoding.EncodingError)` | decodes canonical padded standard Base64 without whitespace |
| function | `Base64UrlEncode(Bytes: ByteStorage.ByteBuffer): Result of (string, Encoding.EncodingError)` | encodes with the URL-safe `-`/`_` alphabet and no padding |
| function | `Base64UrlDecode(Text: string): Result of (ByteStorage.ByteBuffer, Encoding.EncodingError)` | decodes canonical unpadded URL-safe Base64; rejects every `=` |
| function | `HexEncode(Bytes: ByteStorage.ByteBuffer): Result of (string, Encoding.EncodingError)` | emits two uppercase hexadecimal digits per byte |
| function | `HexDecode(Text: string): Result of (ByteStorage.ByteBuffer, Encoding.EncodingError)` | accepts either hex case, even length, and no prefix or separators |

Empty buffers encode as `Ok('')`; empty text decodes as an empty buffer in `Ok`.
Encoders borrow trusted byte storage without rescanning or converting integer elements.
Every decoder rejects whitespace and non-ASCII characters.

## Standard Base64

`Base64Encode` uses the [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648)
standard alphabet `A–Z`, `a–z`, `0–9`, `+`, and `/`. Output length is a multiple
of four. A final group containing one byte has `==`; two bytes have `=`; a
complete three-byte group needs no padding.

`Base64Decode` requires that same alphabet and padding. It rejects omitted,
misplaced, or excess padding, embedded whitespace, and URL-safe alphabet
characters. The unused bits in the final encoded character must be zero:
`'Zg=='` is canonical for byte 102, whereas `'Zh=='` returns
`Error(Encoding.EncodingError.NonCanonical(1))`.

## URL-safe Base64

`Base64UrlEncode` replaces `+` and `/` with `-` and `_` and emits no `=` padding.
For example, bytes `[251, 255]` produce `'-_8'`, compared with standard
Base64 `'+/8='`.

`Base64UrlDecode` accepts only the URL-safe alphabet without padding. Any `=`
is rejected, including otherwise correct standard padding. Length modulo
four may be zero, two, or three; a remainder of one is invalid. Unused final
bits must be zero here too, so `'Zg'` is canonical and `'Zh'` is not. Standard
`+`/`/` characters and whitespace are rejected.

## Hexadecimal

`HexEncode` emits uppercase ASCII, matching the case convention of
[`Std.Text.Format.IntToHex`](format.md#function-inttohexn-integer-digits-integer-string).
Each byte has exactly two digits, including its leading zero when needed.

`HexDecode` accepts uppercase or lowercase digits `0–9`, `A–F`, and `a–f`.
Text must contain an even number of digits. There is no `0x` or `$` prefix,
sign, whitespace, or separator syntax. This decodes binary bytes;
[`Std.Text.Parse.HexInt`](parse.md) parses a signed integer under different rules.

## Errors and offsets

Failures are returned as `Error(Cause)` rather than runtime diagnostics.
Handle them with `case`, propagate compatible errors with `try`, or deliberately
abort with `.Unwrap()`. Error messages are available through `.Message()`.

Every payload field below is an `integer`:

| Variant | Meaning |
|---------|---------|
| `InvalidCharacter(Offset)` | A decoder encountered a byte outside its accepted alphabet. |
| `InvalidLength(Length)` | Input text has an invalid encoded length; `Length` is its UTF-8 byte length. |
| `InvalidPadding(Offset)` | Base64 padding is misplaced or invalid, or URL-safe input contains `=`. |
| `NonCanonical(Offset)` | The final Base64 alphabet character has nonzero unused bits. |
| `LimitExceeded(Limit)` | The binary input or decoded result would exceed the limit; `Limit` is always `1048576`. |

Integer-to-byte validation belongs to `ByteStorage.FromArray`. Decoder offsets are
zero-based **UTF-8 byte positions** in the original text, including for
invalid non-ASCII input; they are not string scalar indexes. For example,
`HexDecode('AAé')` returns `InvalidCharacter(2)`.

## Limits

The maximum encoder input and decoder output is **1_048_576 bytes (1 MiB)**.
The corresponding maximum encoded text lengths, in UTF-8 bytes, are:

| Format | Maximum text length |
|--------|---------------------|
| Padded standard Base64 | 1_398_104 |
| Unpadded URL-safe Base64 | 1_398_102 |
| Hexadecimal | 2_097_152 |

Decoders preflight both encoded length and exact decoded size before allocating
the result. Encoded text within its text-length bound can still exceed the
binary limit. Size violations take precedence over content errors; other
error ordering for input with multiple defects is unspecified.

## Executable examples

```pascal id=example-1 run
program EncodingExamples;

uses Std.Bytes as ByteStorage, Std.Encoding as Encoding, Std.Test as Test;

begin
  Test.AssertEquals('Zm9v', Encoding.Base64Encode( ByteStorage.FromArray([102, 111, 111]).Unwrap()).Unwrap());
  const StandardBytes := Encoding.Base64Decode('Zg==').Unwrap();
  Test.AssertEquals(1, StandardBytes.Length());
  Test.AssertEquals(102, StandardBytes[0]);
  Test.AssertEquals('-_8', Encoding.Base64UrlEncode( ByteStorage.FromArray([251, 255]).Unwrap()).Unwrap());
  const UrlBytes := Encoding.Base64UrlDecode('-_8').Unwrap();
  Test.AssertEquals(2, UrlBytes.Length());
  Test.AssertEquals(251, UrlBytes[0]);
  Test.AssertEquals(255, UrlBytes[1]);
  Test.AssertEquals('00ABFF', Encoding.HexEncode( ByteStorage.FromArray([0, 171, 255]).Unwrap()).Unwrap());
  const HexBytes := Encoding.HexDecode('00abFF').Unwrap();
  Test.AssertEquals(3, HexBytes.Length());
  Test.AssertEquals(0, HexBytes[0]);
  Test.AssertEquals(171, HexBytes[1]);
  Test.AssertEquals(255, HexBytes[2]);
  Test.AssertTrue(Encoding.Base64Decode('Zh==') is Error(Encoding.EncodingError.NonCanonical(1)));
  Test.AssertTrue(ByteStorage.FromArray([0, 256]) is Error(ByteStorage.BufferError.InvalidByte(1, 256)));
  Test.AssertTrue(Encoding.Base64UrlDecode('Zg==') is Error(Encoding.EncodingError.InvalidPadding(2)));
end.
```

To encode a string as binary text, convert it to UTF-8 bytes explicitly with
[`Std.Text.Utf8.Encode`](../network/utf8.md). Decoding Base64 or hex returns
bytes; it does not require them to be valid UTF-8.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime codecs | [`encoding/`](../../../../crates/fpas-runtime/src/stdlib/encoding) |
| Public signatures | [`std_registry/loaded/encoding.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/encoding.rs) |
| Intrinsic IDs | [`intrinsic/encoding.rs`](../../../../crates/fpas-code/src/intrinsic/encoding.rs) |
| Pascal regressions | [`tests/stdlib/encoding/`](../../../../tests/stdlib/encoding) |

## See also

- [Text and parsing](README.md)
- [Conversions](format.md)
- [UTF-8](../network/utf8.md)
