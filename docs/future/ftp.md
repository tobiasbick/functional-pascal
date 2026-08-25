# Future: FTP and FTPS

> Deferred. No implementation is scheduled.

FTP and FTPS may later build on `Std.Net`. Neither protocol is currently implemented, and this plan
does not commit the project to adding them.

## Possible scope

- FTP control connections with bounded reply parsing.
- Active and passive data connections with explicit lifecycle management.
- File transfer and directory operation helpers.
- Explicit and implicit FTPS using the existing TLS transport where it fits the protocol.
- Focused interoperability, timeout, and hostile-input tests.

The control and data channels need a concrete ownership model before implementation starts.
Implemented behavior belongs under `docs/pascal/`, not in this file.
