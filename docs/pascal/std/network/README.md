# Networking

Blocking TCP/TLS transports and HTTP protocol helpers whose parsing and serialization are mostly
implemented in FPAS.

| Unit | Description |
|------|-------------|
| [`Std.Net.Sockets`](net.md) | Hosted TCP/TLS connections and listeners with byte I/O |
| [`Std.Net.Server`](server.md) | Server task/listener ownership, signals, common shutdown budget, and explicit process escalation |
| [`Std.Net.Uri`](uri.md) | Absolute URI parsing, percent-encoded components, and ordered query parameters |
| [`Std.Net.Http`](http.md) | HTTP/HTTPS client and server helpers plus SSE decoding |

## Runnable examples

The [`examples/network/`](../../../../examples/network/README.md) directory
contains paired HTTP and raw TCP programs, HTTPS client/server setup, streaming
response reads, incremental SSE decoding, URI parsing, and UTF-8 conversion.
The examples use bounded request counts and explicit timeouts where applicable.

## See also

- [UTF-8 text encoding](utf8.md)
- [OpenAI-compatible chat](../ai/openai.md)
- [Standard library index](../README.md)
