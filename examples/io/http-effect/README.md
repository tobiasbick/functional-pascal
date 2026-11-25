# Fetch HTTP text with an effect

`Fetch` stores the request address. `Run` supplies a reusable `HttpClient`
and a per-request cancellation token. The client caps buffered responses at
64 KiB. The effect owns and disposes the response, and returns either the
body, an HTTP status failure, or a request failure. A canceled request
propagates through the task and is handled by the program.

Run the example after the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application).

```sh
fpas run examples/io/http-effect/HttpEffect.fpasproj -- https://example.com/
```

The program prints the text from a successful response. A rejected response
prints its numeric status and exits with code 1. The SDK test uses a local
HTTP server for both cases and checks the request deadline.

## Source files

- [HttpEffect.fpasproj](HttpEffect.fpasproj)
- [main.fpas](main.fpas)
