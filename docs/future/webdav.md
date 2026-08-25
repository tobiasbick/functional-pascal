# Future: WebDAV

> Deferred. No implementation is scheduled.

WebDAV may later build on the FPAS HTTP client and server modules. Standard HTTP extension methods,
including `PROPFIND`, can already be expressed through `Request.Create`; WebDAV-specific request,
response, and XML handling are not implemented.

## Possible scope

- WebDAV request and response types on top of `Std.Http`.
- Bounded XML parsing and serialization for WebDAV bodies.
- Client helpers for resource discovery and file operations.
- Server-side routing for the supported WebDAV methods.
- Focused interoperability and hostile-input tests.

This plan should be refined only if WebDAV becomes an active implementation target. Implemented
behavior belongs under `docs/pascal/`, not in this file.
