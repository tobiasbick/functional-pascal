# Networking

Blocking TCP/TLS transports and HTTP protocol helpers whose parsing and serialization are mostly
implemented in FPAS.

| Unit | Description |
|------|-------------|
| [`Std.Net`](net.md) | Hosted TCP/TLS connections and listeners with byte I/O |
| [`Std.Net.Uri`](uri.md) | Absolute HTTP/HTTPS URI parsing |
| [`Std.Net.Utf8`](utf8.md) | UTF-8 encoding and validated decoding |
| [`Std.Http`](http.md) | HTTP/HTTPS client and server helpers plus SSE decoding |

## Runnable examples

The [`examples/network/`](../../../../examples/network/README.md) directory
contains paired HTTP and raw TCP programs, HTTPS client/server setup, streaming
response reads, incremental SSE decoding, URI parsing, and UTF-8 conversion.
The examples use bounded request counts and explicit timeouts where applicable.

## See also

- [OpenAI-compatible chat](../ai/openai.md)
- [Standard library index](../README.md)
