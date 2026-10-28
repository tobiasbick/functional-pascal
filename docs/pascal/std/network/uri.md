# `Std.Net.Uri`

Pure FPAS parsing of absolute HTTP/HTTPS URIs, percent-encoded components, and
ordered query parameters. Import `uses Std.Net.Uri as Uri;` and call public
functions through that alias. These helpers perform no network access.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Uri` | `Scheme`, `Host`, `Port`, and request `Path` |
| type | `UriError` | failures from absolute URI `Parse` |
| type | `QueryParam` | `Name: string` and `Value: Option of string := None` |
| type | `ComponentError` | percent escape, UTF-8, query parameter, and size failures |
| function | `Parse(Text: string): Result of (Uri, UriError)` | parses an absolute URI |
| function | `EncodePathSegment(Text: string): Result of (string, ComponentError)` | percent-encodes one path segment as UTF-8 |
| function | `DecodePathSegment(Text: string): Result of (string, ComponentError)` | percent-decodes once and validates UTF-8 |
| function | `EncodeQueryComponent(Text: string): Result of (string, ComponentError)` | percent-encodes one query name or value |
| function | `DecodeQueryComponent(Text: string): Result of (string, ComponentError)` | decodes once; literal `+` stays `+` |
| function | `ParseQuery(Text: string): Result of (array of QueryParam, ComponentError)` | parses a query body without a leading `?` |
| function | `BuildQuery(Parameters: array of QueryParam): Result of (string, ComponentError)` | builds an ordered query body without a leading `?` |

## Absolute URI parsing

`Parse` recognizes `http` and `https`, applies default ports 80 and 443, accepts bracketed IPv6
hosts, retains query text in `Path`, discards URI fragments before producing an HTTP request
target, and rejects user information and invalid ports. Only HTTP and HTTPS are accepted,
including when a port is explicit. Explicit ports require a nonempty sequence of ASCII decimal
digits in `1..65535`; signs, whitespace, radix prefixes, and digit separators are rejected.

`UriError.InvalidSyntax(Detail)` describes malformed URI structure,
`UnsupportedScheme(Scheme)` retains the rejected scheme, and
`InvalidPort(Detail)` describes an invalid explicit port. The existing `Uri`,
`UriError`, and `Parse` identities and parsing behavior are unchanged. `Parse`
does not automatically decode percent escapes or query components.

## Percent-encoded components

Both encoders leave only ASCII letters, digits, `-`, `.`, `_`, and `~` literal.
Every other UTF-8 byte becomes `%HH` with uppercase hex digits. In particular,
`/` becomes `%2F`, space becomes `%20`, and `+` becomes `%2B`. Encode an
individual path segment or query name/value, then add structural separators
outside that encoded text.

Both decoders accept uppercase or lowercase hex escapes, raw Unicode, and
literal reserved characters. A literal `+` remains `+`; these functions do
not use HTML form encoding. Decoding happens **once**: `%252F` becomes the
literal text `%2F`, not `/`. The resulting bytes must form valid UTF-8. NUL
is allowed and round-trips as `%00`.

## Ordered query parameters

`QueryParam` has a required `Name: string` and an optional
`Value: Option of string := None`. A flag such as `debug` has `None`; `debug=`
has `Some('')`. They remain distinct through parsing and building. Duplicate
names, order, and case are preserved.

`ParseQuery` takes the **query body**, without `?`. It splits on raw `&` and
then on the first raw `=` in each field **before** percent decoding. Thus
`a%26b=c%3Dd` is one parameter with name `a&b` and value `c=d`. Further raw
`=` characters belong to the value. Empty fields between `&` separators are
ignored; `''` and `'&&'` produce empty arrays. A leading `?` or literal `#`
is ordinary component text and is not stripped or treated as a fragment.

Empty names are valid when a value is present: `=x` and `=` produce an empty
name with `Some('x')` and `Some('')` respectively. `BuildQuery` rejects an
empty name with `None`, because it would become an ignored empty field.

`BuildQuery` encodes every name and present value, joins them with `&`, and
adds `=` only for `Some(Value)`. It does not prefix `?`. Its output uses
uppercase escapes even when the parsed input used lowercase escapes or raw
reserved characters. It preserves parameter data rather than the original
spelling of the query.

```pascal id=example-components run
program UriComponentsExample;

uses Std.Net.Uri as Uri, Std.Test as Test;

begin
  Test.AssertEquals('a%2Fb%20%2B', Uri.EncodePathSegment('a/b +').Unwrap());
  Test.AssertEquals('a+b c', Uri.DecodeQueryComponent('a+b%20c').Unwrap());
  Test.AssertEquals('%2F', Uri.DecodePathSegment('%252F').Unwrap());
  const Parameters := Uri.ParseQuery('flag&name=Gr%C3%BC%C3%9Fe&empty=').Unwrap();
  Test.AssertEquals(3, Parameters.Length());
  Test.AssertEquals('flag', Parameters[0].Name);
  Test.AssertTrue(Parameters[0].Value is None);
  Test.AssertEquals('name', Parameters[1].Name);
  Test.AssertTrue(Parameters[1].Value = Some('Grüße'));
  Test.AssertEquals('empty', Parameters[2].Name);
  Test.AssertTrue(Parameters[2].Value = Some(''));
  Test.AssertEquals('flag&name=Gr%C3%BC%C3%9Fe&empty=', Uri.BuildQuery(Parameters).Unwrap());
end.
```

## Component errors and offsets

All six component/query functions return `ComponentError`. The public alias
`Uri.ComponentError` preserves the nominal identity of
`Std.Net.Uri.Components.ComponentError`; it is separate from `UriError`.

| Variant | Fields and meaning |
|---------|--------------------|
| `InvalidEscape` | `Offset: integer`; malformed `%HH` escape at that input position |
| `InvalidEncoding` | `ComponentOffset: integer; Cause: Utf8.Utf8Error`; decoded bytes are not valid UTF-8 |
| `InvalidQueryParameter` | `Index: integer; Reason: string`; an invalid builder record, indexed from zero |
| `LimitExceeded` | `Resource: string; Limit: integer`; a bound in the table below was exceeded |

`InvalidEscape.Offset` is a zero-based **UTF-8 byte offset in the original
input**. Query offsets cover the full query body, including separators and
earlier fields. For example, decoding `é%` reports offset 2, and parsing
`a=ok&name=%` reports offset 10.

`InvalidEncoding.ComponentOffset` is the original input byte position where
the failing name or value begins. It is zero for standalone component
decoding. The retained [`Utf8.Utf8Error`](utf8.md) has its own offset inside
the **decoded component bytes**, not inside the percent-encoded input.
For `first=ok&bad=x%E2%28%A1`, the value begins at input byte 13 and its
invalid continuation is decoded-component byte 2.

Handle these errors as Result data with `case` or compatible `try`
propagation. Use `.Message()` at an output boundary; UTF-8 causes retain their
typed identity rather than becoming strings.

## Limits

Bounds count **UTF-8 bytes**, not Unicode scalars:

| Resource | Limit | Applies to |
|----------|-------|------------|
| `decoded bytes` | 65_536 | encoder text input, decoder component output, and aggregate decoded query names plus present values |
| `encoded bytes` | 196_608 | decoder text input and encoder output, including query `&`/`=` separators for whole-query input/output |
| `query parameters` | 4096 | nonempty parsed fields or builder records |

Each individual component must also fit its component bounds. Empty query
fields do not consume a parameter slot. Query parameter count is preflighted
before percent decoding, so an over-count query returns the count limit even
when an earlier field has a malformed escape.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Uri.fpas`](../../../../lib/Std/Net/Uri.fpas) |
| Component types and errors | [`Uri/Components.fpas`](../../../../lib/Std/Net/Uri/Components.fpas) |
| Regression tests | [`uri_parse_test.fpas`](../../../../tests/stdlib/net/uri_parse_test.fpas) |

## See also

- [Networking index](README.md)
- [`Std.Net.Http`](http.md)
- [UTF-8](utf8.md)
