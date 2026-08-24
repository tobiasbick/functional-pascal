# Networking follow-up

## Completed foundation

The implemented foundation provides VM-owned TCP client connections, URI and UTF-8 helpers, a
non-streaming FPAS HTTP/1.1 client with standard and extension methods, and an OpenAI-compatible
local chat client. Current behavior is documented under
[`docs/pascal/std/network/`](../pascal/std/network/README.md) and
[`docs/pascal/std/ai/`](../pascal/std/ai/README.md).

## Completed HTTPS transport

`Std.Net.ConnectTls` and automatic `https://` selection in `Std.Http.Send` are implemented with
mandatory platform certificate and hostname verification. TCP and TLS share the opaque connection
handle and byte-I/O API; URI handling and the HTTP wire format remain FPAS code.

Verification covers a trusted local TLS round trip, hostname mismatch, untrusted certificates, the
FPAS HTTPS call path, a live public HTTPS request, formatting, Clippy, the workspace build and test
suite, and the complete FPAS regression suite.

## Following work

1. Add streaming response reads and an FPAS Server-Sent Events decoder for streamed chat tokens.
2. Harden the client with bounded redirects, explicit interim-response processing, separate header
   limits, and focused hostile-input tests.
3. Add TCP listener handles and HTTP request/response server modules in FPAS, then reuse the TLS
   transport for HTTPS serving.
4. Build WebDAV helpers and XML handling on the HTTP client/server modules; extension methods such
   as `PROPFIND` already work through `Request.Create`.
5. Build FTP control and data connections on `Std.Net`; reuse TLS for explicit or implicit FTPS.

Each phase requires focused protocol tests, hostile-input limits, docs under `docs/pascal/` only
after implementation, and the standard workspace verification gates.
