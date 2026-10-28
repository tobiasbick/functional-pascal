# FPAS MCP native core

This crate supplies the official MCP SDK adapter and bounded application-call
ownership for the [Std.Ai.Mcp](../../docs/pascal/std/ai/mcp/README.md) library.
It has no ACP or VM dependency. The VM retains callbacks and owns its HTTP
executor; ACP consumes the native server and publication interface for session
attachments. No FPAS value enters the native SDK executor.

## Registration and publication

Build a `Catalog` with validated `Limits`. Register an SDK `Tool` together with
an application-owned handler identity. Input schemas require an object root.
The `jsonschema` validator compiles input and optional output schemas offline;
external file and network references fail registration. Failed registration
does not consume catalog capacity or replace a definition.

`Server::new` consumes the catalog and freezes registration. Each `Endpoint`
implements the SDK's `ServerHandler` and has independent cancellation and reply
rights. The embedding transport supplies authentication, body/connection bounds
and physical shutdown. It must enforce those bounds before handing traffic to
the SDK. The endpoint itself grants no host or application permission.

## Application dispatch

One `Reader` owns reception, including across an asynchronous wait. It receives
validated `Call` values containing the handler identity, arguments, protocol
metadata and a `Responder`. The native adapter executes no application callback
and retains no SDK peer in a detached call.

Apply application policy before effects. `Responder::respond` validates a
successful structured result against its optional output schema and enforces
the result size limit. Invalid output leaves the response right available for
correction. Clones alias one right; a successful reply consumes all aliases.
Dropping the last unconsumed responder settles abandoned work as an error.

Pending capacity covers queued and claimed calls until they terminate. Metadata
counts against both message and retained-payload limits, including metadata the
SDK moves into request context. Cancellation or endpoint closure removes queued
work, invalidates claimed rights and releases admission capacity. A disconnect
cannot undo an application effect and never causes automatic replay.

Closing an endpoint preserves the server and its other endpoints.
`Server::close` stops admission and settles all pending calls. The transport
owner still closes listeners/connections and joins its executor tasks.

## Resource defaults

| Limit | Default |
| --- | --- |
| Tools | 128 |
| Serialized catalog array | 1 MiB |
| Serialized call parameters including metadata, or one result | 1 MiB |
| Pending calls | 64 |
| Retained call and request identity representations | 8 MiB |
| Endpoints | 16 |

## HTTP publication

The native HttpEndpoint::bind operation publishes one independently authenticated
route for a frozen server. Each publication binds a loopback address, receives a
fresh 256-bit bearer credential and carries a distinct endpoint identity.
The URL contains no credential. Supply the confidential authorization header
only to the intended consumer. The HTTP handle has no Debug implementation.

The SDK serves MCP at /mcp with legacy sessions disabled and JSON responses
preferred. Host must match the bound address and port; every present Origin
header is rejected. The
transport rejects missing, incorrect or repeated authorization headers before
reading a request body. Catalog message limits also bound streamed HTTP bodies.
There is no MCP client role, event store, reconnect or replay.

HttpOptions defaults to 127.0.0.1:0, 64 accepted sockets, a 30-second connection
deadline and a five-second shutdown deadline. Incomplete requests count against
socket capacity. Excess connections close before SDK dispatch. Headers are
bounded to 64 fields and a 16 KiB read buffer. Each socket serves one request.
The address must be loopback, connections must be between 1 and 4096, connection
deadlines are at most 300 seconds and shutdown deadlines at most 30 seconds.
Zero deadlines are rejected.

The stop operation revokes access and application response rights immediately.
The close operation joins accepted sockets, aborts them at the shutdown deadline
and reports cleanup failure. Cancelling close retains its join ownership so
another close receives the actual outcome. Dropping the handle revokes access
and aborts its owned listener task; explicit close provides joined cleanup.
Closing a publication preserves its caller-owned server and other publications.
Closing the server retires every publication. Reopening always creates fresh
credentials and never revives retired response rights.

The protocol implementation is pinned to `rmcp` 3.7.0. Its product features have
the server role. The independent wire peers in `tests/` use the production SDK
adapter without enabling the SDK client role.
