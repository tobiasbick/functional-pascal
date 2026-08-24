# `Std.Http`

A non-streaming HTTP/1.1 client implemented in FPAS over `Std.Net`.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Header` | field `Name`, field `Value`, and `Create` constructor |
| type | `Request` | method, URL, headers, byte body, timeout, and response limit |
| static function | `Request.Get/Post/Put/Patch/Delete/Head/Options(Url)` | standard-method request constructors |
| type | `Response` | status, reason, headers, and byte body |
| function | `Send(Request): Result of Response, string` | sends one request over a new connection |
| function | `HeaderValue(Response; Name: string): Option of string` | case-insensitive first match |
| function | `BodyText(Response): Result of string, string` | validated UTF-8 decoding |

Create a request with defaults, then replace the fields needed by the caller:

```pascal
mutable var RequestValue: Request := Request.Create('POST', 'http://127.0.0.1:8080/v1/items');
RequestValue.Headers := [Header.Create('Content-Type', 'application/json')];
RequestValue.Body := Std.Net.Utf8.Encode('{"name":"example"}');
case Send(RequestValue) of
  Ok(ResponseValue): WriteLn(ResponseValue.StatusCode);
  Error(Message): panic(Message)
end
```

Standard methods have short constructors:

```pascal
var GetRequest: Request := Request.Get('http://127.0.0.1:8080/items');
mutable var PutRequest: Request := Request.Put('http://127.0.0.1:8080/items/42');
PutRequest.Body := Std.Net.Utf8.Encode('{"name":"updated"}')
```

`Method` deliberately remains a string so extension methods are not excluded.
For example, a WebDAV request can use
`Request.Create('PROPFIND', 'http://127.0.0.1:8080/documents')`. Method names
must be non-empty RFC 9110 tokens; whitespace, control characters, and token
separators are rejected before a connection is opened.

`Request.Create` defaults to a 30-second timeout and a 16 MiB maximum response. `Send` writes `Host`, `Content-Length`, and `Connection: close`; callers cannot override those fields. It accepts `Content-Length`, chunked, or connection-delimited response bodies. Header injection through CR or LF is rejected.

Responses to `HEAD` have an empty `Body`, even when `Content-Length` describes
the body that an equivalent `GET` would have returned. Informational responses
and status codes `204` and `304` are also treated as bodyless.

Only `http://` transport and origin-form request targets are implemented.
`OPTIONS` addresses a normal URL path; `OPTIONS *` and `CONNECT` authority-form
targets are not implemented. Redirects, compression, proxies, persistent
connections, interim-response processing, streaming, and HTTPS are not handled.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public facade | [`Http.fpas`](../../../../lib/Std/Http.fpas) |
| Connection orchestration | [`Client.fpas`](../../../../lib/Std/Http/Client.fpas) |
| HTTP wire format | [`Wire.fpas`](../../../../lib/Std/Http/Wire.fpas) |
| End-to-end fixture | [`network.rs`](../../../../crates/fpas-cli/src/main_tests/network.rs) |

## See also

- [Networking index](README.md)
- [OpenAI-compatible chat](../ai/openai.md)
