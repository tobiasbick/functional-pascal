# `Std.Http`

An HTTP/1.1 and HTTPS client with buffered and pull-based response bodies, implemented in FPAS over
`Std.Net`.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Header` | field `Name`, field `Value`, and `Create` constructor |
| type | `Request` | method, URL, headers, byte body, timeout, and response limit |
| static function | `Request.Get/Post/Put/Patch/Delete/Head/Options(Url)` | standard-method request constructors |
| type | `Response` | status, reason, headers, and byte body |
| function | `Send(Request): Result of Response, string` | sends and buffers one response |
| type | `StreamResponse` | status, reason, headers, and a `BodyStream` handle |
| function | `OpenStream(Request): Result of StreamResponse, string` | returns after the response headers are available |
| function | `ReadStream(BodyStream; MaxBytes): Result of array of integer, string` | pulls decoded body bytes; an empty array means EOF |
| function | `CloseStream(BodyStream): Result of boolean, string` | closes a response before EOF |
| type | `SseDecoder`, `SseEvent` | bounded incremental Server-Sent Events decoding |
| function | `CreateSseDecoder(MaxEventBytes)` | creates a decoder with a per-event byte limit |
| function | `FeedSse(Decoder; Bytes)` | returns all complete events in one fragment |
| function | `FinishSse(Decoder)` | flushes the final line and finishes the decoder |
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

`Method` deliberately remains a string so extension methods are not excluded. For example, a
WebDAV request can use `Request.Create('PROPFIND', 'http://127.0.0.1:8080/documents')`. Method names
must be non-empty RFC 9110 tokens; whitespace, control characters, and token separators are rejected
before a connection is opened.

`Request.Create` defaults to a 30-second timeout and a 16 MiB maximum response. Requests write
`Host`, `Content-Length`, and `Connection: close`; callers cannot override those fields. Header
injection through CR or LF is rejected.

Plain TCP is selected for `http://` URLs and verified TLS for `https://` URLs. HTTPS certificate and
hostname validation follows the operating system trust policy and cannot be disabled through
`Std.Http`.

Responses to `HEAD` have an empty body, even when `Content-Length` describes the body that an
equivalent `GET` would have returned. Informational responses and status codes `204` and `304` are
also treated as bodyless.

## Streaming responses

`OpenStream` performs the same request validation and TCP/TLS setup as `Send`, but returns after the
response head has been parsed. `ReadStream` decodes `Content-Length`, chunked, and
connection-delimited bodies incrementally and returns at most the requested number of bytes. Thread
the same `BodyStream` handle through each call until an empty array reports EOF. Call `CloseStream`
when abandoning a response early.

```pascal
case OpenStream(Request.Get('https://example.test/events')) of
  Ok(ResponseValue):
  begin
    mutable var Reading: boolean := true;
    while Reading do
    begin
      case ReadStream(ResponseValue.Body, 4096) of
        Ok(Bytes): Reading := Std.Array.Length(Bytes) <> 0;
        Error(Message): panic(Message)
      end
    end
  end;
  Error(Message): panic(Message)
end
```

`MaxResponseBytes` bounds all bytes received for either API, including response headers and chunk
framing. `Send` is implemented by opening and draining the same streaming reader.

## Server-Sent Events

The SSE decoder accepts arbitrary byte fragments, so HTTP read boundaries do not need to match UTF-8
characters, lines, or events. It supports LF, CRLF, and CR line endings; joins repeated `data` fields
with LF; carries the last `id`; defaults an empty event type to `message`; and ignores comments and
unknown fields. `MaxEventBytes` bounds buffered input for one event. `FinishSse` dispatches a pending
final event and rejects later input. The `retry` field is currently ignored because reconnection is
not part of this client.

Only origin-form request targets are implemented. `OPTIONS` addresses a normal URL path;
`OPTIONS *` and `CONNECT` authority-form targets are not implemented. Redirects, compression,
proxies, persistent connections, and interim-response processing are not handled.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public facade | [`Http.fpas`](../../../../lib/Std/Http.fpas) |
| Connection orchestration | [`Client.fpas`](../../../../lib/Std/Http/Client.fpas) |
| Streaming body framing | [`Stream.fpas`](../../../../lib/Std/Http/Stream.fpas) |
| Server-Sent Events decoder | [`Sse.fpas`](../../../../lib/Std/Http/Sse.fpas) |
| HTTP wire format | [`Wire.fpas`](../../../../lib/Std/Http/Wire.fpas) |
| Buffered end-to-end fixture | [`network.rs`](../../../../crates/fpas-cli/src/main_tests/network.rs) |
| Streaming fixtures | [`network_streaming.rs`](../../../../crates/fpas-cli/src/main_tests/network_streaming.rs) |
| HTTPS rejection fixture | [`network_tls.rs`](../../../../crates/fpas-cli/src/main_tests/network_tls.rs) |

## See also

- [Networking index](README.md)
- [OpenAI-compatible chat](../ai/openai.md)
