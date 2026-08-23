# Networking follow-up

## Current foundation

The implemented slice provides VM-owned TCP client connections, FPAS URI/UTF-8/HTTP modules, and an OpenAI-compatible non-streaming chat client. Current behavior is documented under [`docs/pascal/std/network/`](../pascal/std/network/README.md) and [`docs/pascal/std/ai/`](../pascal/std/ai/README.md).

## Planned order

1. Add a native TLS transport with certificate and hostname verification; select TCP or TLS from the URI scheme inside `Std.Http`.
2. Add streaming response support and an FPAS Server-Sent Events decoder for streamed chat tokens.
3. Add TCP listener handles and HTTP request/response server modules in FPAS.
4. Build WebDAV request methods and XML handling on the HTTP client/server modules.
5. Build FTP control and data connections on `Std.Net`; reuse TLS for explicit or implicit FTPS if required.

Cryptography, certificate stores, sockets, and other OS resources remain native. URI handling, message framing, headers, protocol state, WebDAV methods, FTP commands, and model-specific JSON remain FPAS code.

Each phase requires focused protocol tests, hostile-input limits, docs under `docs/pascal/` only after implementation, and the standard workspace verification gates.
