# Network examples

These examples cover the implemented `Std.Net`, `Std.Http`, `Std.Net.Uri`, and
`Std.Net.Utf8` APIs. The OpenAI-compatible chat demo remains in
[`examples/openai-chat/`](../openai-chat/).

## Local HTTP pair

Start the bounded server in one terminal:

```sh
fpas run examples/network/http_server.fpas -- 18080
```

Then start the client in another terminal:

```sh
fpas run examples/network/http_client.fpas -- 18080
```

The client sends a buffered `GET`, a buffered `POST`, and a streaming `GET`.
The server stops after those three requests, so the example does not leave a
listener running.

## Raw TCP echo pair

The TCP pair shows the lower-level connection API used beneath HTTP. Start the
server first, then the client:

```sh
fpas run examples/network/tcp_echo_server.fpas -- 18081
fpas run examples/network/tcp_echo_client.fpas -- 18081
```

Both programs set explicit I/O timeouts, handle partial reads and writes, use
an LF-delimited request, close their connections, and exit after one exchange.

## HTTPS

The HTTPS client uses the operating system trust store and does not provide an
insecure certificate bypass:

```sh
fpas run examples/network/https_client.fpas -- https://example.com/
```

The HTTPS server needs a PEM certificate chain and matching PEM private key. It
serves one request and then exits:

```sh
fpas run examples/network/https_server.fpas -- certificate.pem private-key.pem 18443
```

Use a certificate trusted by the client when testing the pair. The server
example deliberately does not generate or embed private keys.

## Protocol helpers

The remaining examples run without opening sockets:

```sh
fpas run examples/network/sse_decoder.fpas
fpas run examples/network/uri_utf8.fpas
```

`sse_decoder.fpas` feeds one event across multiple byte fragments.
`uri_utf8.fpas` parses an absolute HTTP URI and round-trips Unicode text through
UTF-8 bytes.

## Reference

- [`Std.Net`](../../docs/pascal/std/network/net.md)
- [`Std.Http`](../../docs/pascal/std/network/http.md)
- [`Std.Net.Uri`](../../docs/pascal/std/network/uri.md)
- [`Std.Net.Utf8`](../../docs/pascal/std/network/utf8.md)
