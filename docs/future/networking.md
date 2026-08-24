# Networking follow-up

## Completed foundation

The implemented foundation provides VM-owned TCP client connections, URI and UTF-8 helpers, a
buffered FPAS HTTP/1.1 client with standard and extension methods, and an OpenAI-compatible local
chat client. Current behavior is documented under
[`docs/pascal/std/network/`](../pascal/std/network/README.md) and
[`docs/pascal/std/ai/`](../pascal/std/ai/README.md).

## Completed HTTPS transport

`Std.Net.ConnectTls` and automatic `https://` selection in `Std.Http.Send` are implemented with
mandatory platform certificate and hostname verification. TCP and TLS share the opaque connection
handle and byte-I/O API; URI handling and the HTTP wire format remain FPAS code.

Verification covers a trusted local TLS round trip, hostname mismatch, untrusted certificates, the
FPAS HTTPS call path, a live public HTTPS request, formatting, Clippy, the workspace build and test
suite, and the complete FPAS regression suite.

## Completed response streaming and SSE

`Std.Http.OpenStream`, `ReadStream`, and `CloseStream` provide bounded pull-based response bodies for
`Content-Length`, chunked, and connection-delimited HTTP/HTTPS responses. `Send` drains the same
reader. The FPAS SSE decoder accepts fragmented byte input, carries event IDs, combines data lines,
and enforces a per-event limit. Focused fixtures cover fragmented chunked SSE delivery, truncated
bodies, decoder fragmentation, finish behavior, and the event-size limit.

## Completed HTTP client hardening

The FPAS client now follows a bounded set of HTTP redirects, resolves relative `Location` values,
applies status-specific method rewriting, and strips credentials when the origin changes. Separate
response-head and total-response limits, bounded informational responses, strict response fields,
and unambiguous body framing reject hostile or malformed inputs. Focused fixtures cover these paths.

## Following work

1. Add TCP listener handles and HTTP request/response server modules in FPAS, then reuse the TLS
   transport for HTTPS serving.
2. Build WebDAV helpers and XML handling on the HTTP client/server modules; extension methods such
   as `PROPFIND` already work through `Request.Create`.
3. Build FTP control and data connections on `Std.Net`; reuse TLS for explicit or implicit FTPS.

Each phase requires focused protocol tests, hostile-input limits, docs under `docs/pascal/` only
after implementation, and the standard workspace verification gates.
