# `Std.Net.Uri`

Pure FPAS parsing of absolute HTTP and HTTPS URIs.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Uri` | `Scheme`, `Host`, `Port`, and request `Path` |
| function | `Parse(Text: string): Result of Uri, string` | parses an absolute URI |

`Parse` recognizes `http` and `https`, applies default ports 80 and 443, accepts bracketed IPv6
hosts, retains query text in `Path`, discards URI fragments before producing an HTTP request
target, and rejects user information and invalid ports.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Uri.fpas`](../../../../lib/Std/Net/Uri.fpas) |
| Regression tests | [`uri_parse_test.fpas`](../../../../tests/stdlib/net/uri_parse_test.fpas) |

## See also

- [Networking index](README.md)
- [`Std.Http`](http.md)
