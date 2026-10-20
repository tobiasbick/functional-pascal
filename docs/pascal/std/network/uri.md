# `Std.Net.Uri`

Pure FPAS parsing of absolute HTTP and HTTPS URIs.

## Errors

`UriError.InvalidSyntax(Detail)` describes malformed URI structure,
`UnsupportedScheme(Scheme)` retains the rejected scheme and
`InvalidPort(Detail)` describes an invalid explicit port. Private fallible parser
helpers use the same family. Accepted grammar and validation order are unchanged;
use Message() at an output boundary rather than inspecting diagnostic wording.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Uri` | `Scheme`, `Host`, `Port`, and request `Path` |
| function | `Parse(Text: string): Result of (Uri, UriError)` | parses an absolute URI |

`Parse` recognizes `http` and `https`, applies default ports 80 and 443, accepts bracketed IPv6
hosts, retains query text in `Path`, discards URI fragments before producing an HTTP request
target, and rejects user information and invalid ports. Only HTTP and HTTPS are accepted,
including when a port is explicit. Explicit ports require a nonempty sequence of ASCII decimal
digits in `1..65535`; signs, whitespace, radix prefixes, and digit separators are rejected.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Uri.fpas`](../../../../lib/Std/Net/Uri.fpas) |
| Regression tests | [`uri_parse_test.fpas`](../../../../tests/stdlib/net/uri_parse_test.fpas) |

## See also

- [Networking index](README.md)
- [`Std.Http`](http.md)
