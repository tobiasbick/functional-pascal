# Proposed client interface

Package: [AP50](README.md). These are proposed library contracts, not existing
FPAS declarations. The numbered work packages own their implementation.

## Module layout

| Public module | Responsibility |
| --- | --- |
| `Std.Acp` | Stable connection operations, event reception and request replies |
| `Std.Acp.Types` | Stable v1 requests, responses, notifications, content, capabilities and identifiers |
| `Std.Acp.Host` | Opt-in filesystem, terminal and interactive-login adapters |
| `Std.Acp.Experimental` | Experimental v1 requests, fields and updates |
| `Std.Acp.V2` | Draft v2 requests, responses, updates and send/reply operations |

Keep native registration and conversion split by concern. Native intrinsic
unit lookup must recognize these full names; ordinary `uses ... as ...`
parsing and source nominal identity remain unchanged. Each unit receives its
generated editor declaration. Do not create a second runtime protocol engine
in FPAS or expose Rust executors, futures, traits or thread identifiers.

## Values and handles

`Connection`, `RequestToken` and `Responder` are opaque, connection/VM-owned
handles. Copies alias their state. They cannot be forged from integers or used
in another VM. A session ID is protocol data, not proof of a live session or
authorization. Session and request IDs remain case-sensitive opaque values.

Protocol records and data enums are ordinary detached FPAS values. They can
cross existing channels and be inspected by the debugger. They contain no
borrowed Rust memory or callback-owned VM cells. `RawJson` is a value record
containing JSON text; validate it at every native boundary even when produced
through `ParseRawJson`. It is not an executable callback or a live resource.

Type mapping is mechanical: signed 64-bit integers become `integer`, bounded
unsigned values that fit become validated `integer`, and full-width `u64`
becomes `UInt64` with canonical decimal digits. Provide checked construction,
decimal formatting and conversion to/from `integer`. No u64 value passes
through `real`. Finite binary64 fields use `real`; IDs use nominal string
types. Arrays and objects retain their ordering.

Use `Option` where the pinned schema equates absence and null. Use
`Field of T` with `Missing`, `NullValue` and `Value(T)` where a patch distinguishes
them. Open enums retain an `Unknown(Tag, RawJson)` variant when payloads exist,
or an unknown string for string-only enums. An unknown mode is never treated
as a known mode. Preserve custom metadata and arbitrary protocol error data.

`NullValue` represents explicit JSON null; `null` itself is a reserved FPAS keyword.

## Connection setup

`DefaultOptions()` returns explicit defaults. `ConnectProcess(Launch, Options)`
starts the configured executable without shell interpolation and returns a
connection in `Uninitialized` state. `Launch` contains command, arguments,
optional cwd and environment overrides with the same validation as `Std.Proc`.
Opening the process does not authenticate, create a session or grant access.

Options contains ClientInfo, ClientCapabilities, SupportedVersions,
ExperimentalOptions and the configurable limits below. SupportedVersions
defaults to `[V1]`; initialization requests the highest selected version.
V2 must be selected explicitly. A response selecting a supported lower version
sets the negotiated version and returns its typed initialization outcome;
subsequent work uses that version's explicit interface. An unsupported selected
version fails negotiation and closes the connection without creating work.

`ConnectMessages(Options)` returns a connection and a production transport
endpoint. The application moves complete JSON-RPC messages using
`FeedMessage(Endpoint, RawJson)`, `TryTakeMessage(Endpoint)` and
`CloseInput(Endpoint)`. Accepted input is bounded and owned before return.
`Ok(false)` on feed means full and accepts nothing; an empty outbound result
means pending. Closing input delivers EOF after admitted input. A successful
outbound take consumes exactly one frame. This supports real custom links
and deterministic in-process peers without a test-only injection interface.

The subprocess adapter uses newline-delimited UTF-8 JSON, handles partial
reads/writes, drains stderr separately, and bounds unfinished frames. It must
not use LSP Content-Length framing. A chunk bound is not a total message bound.

## Stable operations

These signature shapes fix the ownership and error contracts. `AcpError` is
the nominal operational error family. Concrete types live in `Std.Acp.Types`.

| Operation | Result and meaning |
| --- | --- |
| `Send(Connection, ClientRequest)` | `Result of (RequestToken, AcpError)`; validates and admits one typed request |
| `Notify(Connection, ClientNotification)` | `Result of (boolean, AcpError)`; true after local admission |
| `TryReceive(Connection)` | `Result of (Option of Event, AcpError)`; consumes at most one event without waiting |
| `Receive(Connection, TimeoutMillis, CancellationToken)` | `Result of (Event, AcpError)`; cooperative wait that does not block a VM worker |
| `Respond(Responder, ClientResponse)` | `Result of (boolean, AcpError)`; validate the response type before consuming the responder |
| `RespondError(Responder, ProtocolError)` | Same local result; publishes a peer error response |
| `CancelRequest(Connection, RequestToken)` | `Result of (boolean, AcpError)`; request-local cancellation, with false for an already completed token |
| `State(Connection)` | Detached version, capabilities and lifecycle snapshot |
| `Close(Connection)` | `Result of (boolean, AcpError)`; idempotent cooperative close and native cleanup |
| `Release(Connection)` | `Result of (boolean, AcpError)`; close if needed, discard queued events and invalidate all aliases |

Receive timeout and cancellation parameters are `Option of integer` and
`Option of Tasks.CancellationToken`; None means no corresponding bound. A
negative timeout is invalid and Some(0) probes immediately after validation
and cancellation checks. The default/validation rules also apply to Connect
options and Host operations carrying these parameters.

`Send` returns after local admission, not remote completion. `Event` contains
Payload and RawMessage fields. RawMessage is `Some(RawJson)` for a received wire
message and None for a synthetic local event, preserving fields the SDK ignores.
Payload includes
`Response(Token, AgentResponse)`, `RequestFailed(Token, AcpError)`,
`Notification(AgentNotification)`, `Request(AgentRequest, Responder)`,
`RequestCancelled(Responder)`, `UnknownNotification(Method, RawJson)` and
`Closed(Cause)`, plus distinct v2 payload variants. V2 initialization uses
`V2Initialized(Token, InitializeOutcome)` instead of an ordinary response,
including the SDK's typed lower-version response. A peer JSON-RPC error is
`RequestFailed(Token, AcpError.Peer(ProtocolError))`; keep its code, message and
optional data. Known terminal results of sent requests are never silently lost.
There is no second consumable result stored behind the token.

Exactly one connection reader may be receiving at a time; a concurrent reader
gets `Busy`. Sending, cancellation and answering from other tasks are supported.
Copying a connection does not copy or broadcast events. Applications can fan
out detached events through ordinary FPAS channels.

Responders are single-use across all aliases. A wrong response variant returns
`InvalidArgument` without consuming the responder. Valid response admission
consumes it atomically. A repeated/expired/foreign responder returns `Closed`
or `InvalidArgument` before publication. Request cancellation expires it and
produces its cancellation event even while the application is deciding.

## Ordering and cancellation

The native driver preserves admitted outbound order and received event order
per connection. It spawns peer-waiting work outside the SDK's ordered callback
loop. A slow permission dialog must not block another session, cancellation,
request responses or connection teardown. All necessary local registrations
exist before publishing new/load/resume requests, including replay routing.

Waiting timeout/cancellation stops only the receive operation and consumes no
event. It does not cancel a prompt. `session/cancel` is a typed notification;
it also answers pending permissions for that session with the protocol's
cancelled outcome. `CancelRequest` sends `$/cancel_request` for a published
request. Both retain the original request until its actual result or terminal
connection failure. Late results are settled once; they are not reassigned.

## Defaults and bounds

| Resource | Default | Allowed configuration |
| --- | --- | --- |
| Connections per VM | 16 | Fixed initial bound |
| Pending outgoing requests per connection | 128 | 1..4096 |
| Pending incoming responders per connection | 128 | 1..4096 |
| Retained sessions per connection | 64 | 1..4096 |
| Terminals per connection, including exited unreleased terminals | 16 | 1..256 |
| MCP routes per connection, across all scopes | 128 | 1..4096 |
| Received events | 256 entries and 16 MiB aggregate | 1..4096 entries; 1..64 MiB aggregate |
| Outbound admission | 128 entries and 16 MiB aggregate | Same configurable ceilings |
| Complete message including JSON envelope | 16 MiB | 1 KiB..64 MiB |
| Retained agent stderr | 256 KiB | 0..4 MiB |
| Shutdown grace before process-group termination | 2000 ms | 0..60000 ms |
| Experimental features | All false | Individual booleans |
| Retained output per terminal | 1 MiB | 0..16 MiB |
| Aggregate reserved terminal output per connection | 16 MiB | 0..64 MiB |

Reserve session capacity before publishing new/load/resume/fork operations that
retain a new session. Observed restricted sessions count too; listing detached
session descriptions does not. Release a reservation on failed setup or when
its retained session state is removed. Reserve route capacity before exposing
an endpoint. Reserve terminal count and its requested output budget before
launch; retain both reservations after exit/kill until release. Aggregate output
includes retained UTF-8 and decoder carry buffers. Reject exhaustion with
`LimitExceeded` before publication or host effects, and return reservations
exactly once during release/teardown. Locally initiated growth is rejected;
unavoidable inbound growth beyond a bound fails the connection explicitly.

Count retained native and FPAS representations in byte budgets, including
detached pending responders, inbound/outbound frames and extension payloads.
An outbound bound failure returns `LimitExceeded` without publishing anything.
An inbound queue/frame violation fails the connection explicitly; never drop
content, auto-approve, or stall a full ordered callback queue indefinitely.
The terminal connection error and settlement of already admitted requests use
reserved state, independent of the full event queue. Drain admitted events,
then synthetic request failures, then the single `Closed` event. Further reads
return `Closed`. Stderr truncation has an explicit truncation marker.

## Host and interaction contract

Capabilities default to unsupported until the application registers a provider
or explicitly handles the corresponding events. The library implements every
client capability; an application may grant a subset. Permission and elicitation
requests always reach application code. No policy silently selects allow-always,
opens URLs, runs a login command, reads a file or starts a terminal.

`Std.Acp.Host` supplies explicit operations for disk read/write, terminal
create/output/wait/kill/release and interactive login. Applications invoke
them after their policy check, or implement the request themselves. Request
helpers receive the original Responder and matching typed request, checking
that it is still active before admitting an effect. Host calls return typed
protocol responses that are passed to `Respond`; they do not consume the
responder behind the application's back. Cancellation after admission cannot
undo completed I/O, but cancellation before admission prevents the effect.

Host execution has a separate atomic single-use right per Responder. Method,
identity and argument validation failures before admission leave this right
available. Once admitted, even a failed operation cannot be repeated with that
Responder. A concurrent helper call returns `Busy`; a subsequent call after
completion returns `InvalidArgument`, without repeating the effect. Responding
while the helper is running returns `Busy` without consuming the response
right. After helper completion the application may send its response or map
its error through `RespondError`. Cancellation/close can expire the response
right while an already admitted effect finishes under owned cleanup.

Disk reads obey one-based line ranges. Editor applications can instead answer
from unsaved buffers. Terminal adapters retain bounded combined output, preserve
UTF-8 boundaries on truncation and retain final output after process exit.
Terminal identity is scoped to connection and session. Killing retains the
terminal; releasing it terminates any remaining command and frees ownership.

Terminal authentication runs the same configured agent command, base args,
cwd and environment with the method's appended args and overrides. It runs
separately with interactive I/O, then requires reconnect and reinitialization.
Never pass a terminal method to protocol `authenticate`. URL elicitation emits
the destination and agent identity for host display and consent; completing
the URL flow uses the protocol completion notification.

## Errors, shutdown and version separation

Local `AcpError` variants include `InvalidArgument`, `UnsupportedCapability`,
`UnsupportedVersion`, `Busy`, `Closed`, `Timeout`, `Cancelled`, `LimitExceeded`,
`InvalidMessage`, `Host(HostError)` and `Peer(ProtocolError)`. Errors identify
the operation. Ordinary diagnostics exclude prompts, JSON bodies, credentials,
environment values and filesystem contents; retain detailed returned error data
for the application to handle deliberately.

`Close` rejects new admission, cancels responders, settles pending requests and
reaps owned processes, pipes and driver tasks. It invalidates connection aliases
for new work, while terminal events remain drainable. The bounded closed event
state retains its connection-handle slot until `Release`; it retains no live
process, responder or executor task. `Release` explicitly discards any unread
events and invalidates the handle and every alias. Applications use it in their
final cleanup even after a failed connection. A cleanup host failure
is reported and ownership remains with VM teardown. VM normal completion,
runtime failure and debugger disconnect execute the same cleanup. Existing
seeded-debugger host-effect restrictions apply; do not invent deterministic
replay guarantees for external agents.

V2 uses `Std.Acp.V2.Send/Notify/Respond` and its own request/response types with
the same connection, event-consumption and lifetime rules. The connection's
event union has a versioned payload. V1 payloads cannot be sent on v2 or vice
versa. V2 prompt acceptance is distinct from later idle/completion updates.
No transparent retry of a prompt or automatic protocol fallback may duplicate
effects. A lower-version initialization outcome is returned explicitly; the
application may then select the supported v1 contract before creating work.
