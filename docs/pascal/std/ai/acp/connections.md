# Connections and transports

Import `Std.Acp as Acp` and `Std.Acp.Types as Types`. A connection starts in
`Lifecycle.Uninitialized`. Send an explicit typed initialize request before
using stable session methods. See [sessions](sessions.md) for negotiation,
capability checks and prompt ownership. [Host adapters](host.md),
[experimental options](experimental.md) and [v2](v2.md) describe their policy.

## Setup

| Operation | Result |
| --- | --- |
| `DefaultOptions()` | `Options` |
| `ConnectMessages(Options)` | `Result of (MessageConnection, Types.AcpError)` |
| `ConnectProcess(Launch, Options)` | `Result of (Connection, Types.AcpError)` |
| `FeedMessage(Endpoint, Message)` | `Result of (boolean, Types.AcpError)` |
| `TryTakeMessage(Endpoint)` | `Result of (Option of Types.RawJson, Types.AcpError)` |
| `CloseInput(Endpoint)` | `Result of (boolean, Types.AcpError)` |

`MessageConnection` contains `Connection` and `Endpoint`. Feed accepts one
complete UTF-8 JSON-RPC message and owns its bounded copy before returning
`Ok(true)`. `Ok(false)` accepts nothing because input capacity is occupied.
An empty outbound take means pending; a successful take consumes one frame.
`CloseInput` delivers EOF after admitted input and returns false on repetition.
JSON-RPC batches are rejected as invalid messages.

`Launch` contains `Command`, `Arguments`, `WorkingDirectory` and `Environment`.
An environment entry is `Acp.EnvironmentVariable(Name, Value)`; `None` removes
the inherited variable. Launch uses the validation and process-group ownership
of [Std.Proc](../../host/proc.md). It starts the executable directly.
Client metadata budgets and connection/driver capacity are validated and reserved
before starting the process. Failed admission starts no child; abandoned process
transports terminate and reap their child and return the process-registry slot.

The process transport exchanges newline-delimited JSON. It handles fragmented
UTF-8 and partial writes, drains stderr separately, and rejects an unfinished
nonempty final frame. Complete preceding frames remain deliverable. A chunk
size never replaces the complete-message limit.

`Options` contains `ClientInfo`, `ClientCapabilities`, `SupportedVersions`,
`ExperimentalOptions`, `Limits` and `V2ClientCapabilities`. The default is `[Acp.ProtocolVersion.V1]`,
absent client information, unsupported client capabilities, and false
experimental flags. V2 payload dispatch requires explicit V2 selection.
Experimental flags are independent opt-ins and also require the corresponding
peer capabilities. Opening a connection grants no filesystem or terminal access.

The explicit initialize request must match the configured `ClientInfo`,
`ClientCapabilities` and highest supported version. Configure the options
before connecting, then include the same values in `Types.InitializeRequest`.
Changing the offered capabilities after opening a transport is rejected.

## Messages and events

| Operation | Result |
| --- | --- |
| `Send(Connection, Types.ClientRequest)` | `Result of (RequestToken, Types.AcpError)` |
| `Notify(Connection, Types.ClientNotification)` | `Result of (boolean, Types.AcpError)` |
| `TryReceive(Connection)` | `Result of (Option of Event, Types.AcpError)` |
| `Receive(Connection, TimeoutMillis, CancellationToken)` | `Result of (Event, Types.AcpError)` |
| `Respond(Responder, Types.ClientResponse)` | `Result of (boolean, Types.AcpError)` |
| `RespondError(Responder, Types.ProtocolError)` | `Result of (boolean, Types.AcpError)` |
| `CancelRequest(Connection, RequestToken)` | `Result of (boolean, Types.AcpError)` |
| `State(Connection)` | `Result of (ConnectionState, Types.AcpError)` |

Send/notify return after local admission. SDK-assigned wire IDs remain separate
from opaque request tokens. `Std.Acp.V2.Send`, `Notify` and `Respond` use the
separate V2 payload types; a family mismatch fails before admission.

Each request and responder retains its admission family until it settles.
A lower-version negotiation changes the family for new work; it does not
reinterpret outstanding values or trigger a resend.

`Event` contains `Payload: EventPayload` and `RawMessage: Option of Types.RawJson`.
The raw document retains ignored envelope/payload fields and exact JSON number
spellings. Events include `Response(Token, Payload)`, `RequestFailed(Token, Cause)`,
`Notification(Payload)`, `Request(Payload, Responder)`, `RequestCancelled(Responder)`,
`UnknownNotification(Method, Payload)` and `Closed(Status)`. V2 data uses the
corresponding `V2Response`, `V2Notification` and `V2Request` variants. The
`V2Initialized` carries the separately typed initialization outcome. MCP carrier
requests have separate `McpResponse` and `V2McpResponse` outcomes because the
pinned SDK omits their reverse-direction root variants; see [MCP endpoints](experimental.md#mcp-carrier-endpoints).

Exactly one reader may own reception, including while suspended. Another
reader gets `Busy`. Timeout is `Option of integer`, cancellation is
`Option of Tasks.CancellationToken`; `None` disables that bound. Negative
timeouts fail validation. A wait checks validation, cancellation, readiness,
then timeout. Timeout/cancellation consumes no event and does not cancel a
remote request. Main execution, scheduled tasks and the ordinary debugger
use published waits; a pool worker remains available to other tasks.

Incoming requests retain an SDK response right without holding its callback
open. The application can defer a reply while other messages progress. A
wrong response variant leaves the right available. Valid admission consumes
it once across aliases. Reuse or cancellation returns `Closed` before another
reply is published. Peer errors retain code, message and arbitrary data.

`CancelRequest` admits `$/cancel_request` in outbound order and returns false
for an already completed/cancelled token. The request remains pending until
its actual response or connection termination. Incoming cancellation expires
the responder and emits `RequestCancelled` in received order.

`ConnectionState` contains `Lifecycle`, `RequestedVersion`, `NegotiatedVersion`,
`NegotiatedVersion`, `AgentCapabilities`, `AgentInfo` and `AuthMethods`, plus
separate `V2AgentCapabilities`, `V2AgentInfo` and `V2AuthMethods`. Negotiated values remain
absent until successful initialization; auth methods start as an empty array.
The original typed response and its raw message remain available through the
response event. Closing retains the bounded negotiation snapshot for diagnosis.

## Bounds and cleanup

There are 16 connection slots per VM. A closed connection retains its slot
until release. All limits are validated before starting a process.

| `Limits` field | Default | Allowed |
| --- | --- | --- |
| `OutgoingRequests`, `IncomingRequests` | 128 each | 1..4096 |
| `Sessions` | 64 | 1..4096 |
| `Terminals` | 16 | 1..256 |
| `Routes` | 128 | 1..4096 |
| `QueueEntries` | 256 | 1..4096 |
| `OutboundEntries` | 128 | 1..4096 |
| `QueueBytes` | 16 MiB | 1..64 MiB |
| `MessageBytes` | 16 MiB | 1 KiB..64 MiB |
| `StderrBytes` | 256 KiB | 0..4 MiB |
| `ShutdownMillis` | 2000 | 0..60000 |
| `TerminalBytes` | 1 MiB | 0..16 MiB |
| `AllTerminalBytes` | 16 MiB | 0..64 MiB |

Queues account for retained raw/detached payloads and SDK publication work.
Retained request/responder metadata has bounded reservations too. A full
client-state budget reports `LimitExceeded('ClientStateBytes', ...)`; it uses
`QueueBytes` as its separate bound for negotiated data and session modes/config.
Session slots include pending setup requests. A full
outbound admission queue rejects publication. A full event queue fails the
connection visibly. Terminal request failures and the final close event use
reserved state: drain admitted events, then unanswered-request failures, then
exactly one `Closed`. Further receives return `Closed`.

`Close(Connection)` cooperatively finishes cleanup and returns true only for
the first local close. It rejects new work immediately and remains usable
when physical output is blocked. It closes stdin, allows the shutdown grace,
then terminates/reaps the process group. `Release(Connection)` closes if needed,
discards unread events, returns the slot and invalidates connection/endpoint
aliases. Both return `Result of (boolean, Types.AcpError)`.
If physical cleanup fails, Close/Release report the host error and retain the
connection slot and process ownership for VM teardown.

`ClosedStatus` contains `Cause`, `ProcessExit`, `Stderr` and `StderrTruncated`.
`ProcessExit` is absent for a message link, or contains `Exited(Code)`,
`Signaled(Signal)` or `Unknown`. Windows exit codes preserve all 32 bits.
Stderr remains bounded while excess output is drained. VM completion, runtime
failure and debugger disconnect close all owned connections and children.
External connections retain the debugger's host-effect restrictions; seeded
execution does not replay agents.

## Implementation

The [native driver](../../../../../crates/fpas-acp/src/connection/mod.rs) owns
SDK dispatch and bounded detached events. The
[VM bridge](../../../../../crates/fpas-runtime/src/execution/vm/hosted/acp/mod.rs) constructs
FPAS values after dequeue. Its
[process adapter](../../../../../crates/fpas-runtime/src/execution/vm/hosted/proc/acp.rs)
reuses the existing child registry. The
[production message regression](../../../../../tests/stdlib/acp/connection_test.fpas)
also runs with one worker in the VM tests.
