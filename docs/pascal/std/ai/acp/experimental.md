# `Std.Acp.Experimental`

Typed v1 extensions from the pinned ACP SDK and schema. These declarations
cover provider configuration, fork operations, MCP routing and the supporting
experimental session, plan and subagent values. Their presence grants no
capability and enables no host effect.

Import `uses Std.Acp.Experimental as Exp;` alongside `Std.Acp.Types`.
The stable root enums include the corresponding extension variants with
payloads owned by this unit. The v1 payload codecs in `Std.Acp.Types` handle
these variants with the same validation and raw-data preservation rules.

Generated [editor declarations](../../../../../lib/api/Std/Acp.Experimental.fpas)
list every record and enum from the current pin.

## Independent opt-ins

Set `Options.ExperimentalOptions` before connecting. All seven booleans default
to false. Enabling one does not enable the others or grant filesystem/process
access. Methods also require the pinned peer capability, where one exists.

| Option | Behavior |
| --- | --- |
| `TokenUsage` | Expose exact prompt-end/idle usage counters; disabled counters remain in `Event.RawMessage` |
| `Providers` | Typed `providers/list`, `providers/set`, `providers/disable`; requires peer `providers: {}` |
| `McpOverAcp` | Explicit bounded carrier endpoints; requires peer MCP-over-ACP support |
| `PlanOperations` | Experimental ID-addressed plan updates/removal; v1 also advertises `ClientCapabilities.Plan` |
| `Fork` | `session/fork` requires peer fork support and a client-owned parent |
| `SystemPrompt` | Client system-prompt fields require peer support; override additionally requires override support |
| `Subagents` | Observe restricted child sessions and inter-session messages; v1 also advertises `ClientCapabilities.Subagents` |

Disabled inbound experimental updates remain unknown notifications with their
original raw document. They do not activate optional session state. Stable
usage updates and stable v2 plan updates remain available independently.
Observed subagents consume bounded session capacity. Observation does not grant
prompt, configuration, fork, deletion or close rights. The pinned per-child
capability can explicitly permit cancellation. Omission retains that capability;
null clears it, and an object replaces the whole set. The parent association
remains fixed. Clearing a state snapshot does not imply idle.

Fork requests preserve the schema's empty/null/default semantics. In this pin,
missing, empty or null `mcpServers` and additional roots do not inherit the
parent's configuration. The library does not insert an inherited system prompt
or retry a failed provider operation. Provider headers, bodies and peer-error
data remain available to the caller; native debug formatting redacts raw JSON.

## MCP carrier endpoints

`Acp.RegisterMcpEndpoint(Connection, ServerId, SessionId)` returns an opaque
`McpEndpoint`. `SessionId` is optional; `None` gives connection lifetime.
Registration requires an initialized connection and both MCP gates. Server IDs
are unique within that connection. The `Routes` and queue-byte limits bound
retained endpoints and carrier requests. Copies alias the same endpoint.

The application advertises the matching `McpServerAcp` descriptor when setting
up a session, then handles typed `MessageMcpRequest` events and responds with
the matching `MessageMcpResponse`. Registration executes no tool. There is no
automatic proxy, tool server, nested transport, or filesystem authority.

Use `Acp.SendMcp(Endpoint, Exp.MessageMcpRequest)` for a reverse-direction
request. V2 uses `V2.SendMcp` and its own supporting DTO. Both return an ordinary
request token; results arrive as `EventPayload.McpResponse` or `V2McpResponse`.
The pinned SDK exposes these supporting DTOs but omits reverse MCP variants
from `ClientRequest`/`AgentResponse`. These typed operations use its generic
connection rather than inventing a raw extension or a second JSON-RPC engine.

The carrier keeps outer ACP IDs, inner MCP IDs, target server ID and opaque
body separate. Identical inner IDs on distinct endpoints do not collide.
`RequestCancelled` uses the existing responder/token path. Waiting for an inner
result does not stop unrelated connection events.

A session-scoped endpoint can be registered before load/resume to serve replay.
It becomes usable while that session is setting up and retires on successful
session close. Its descriptor must match that session. Fork accepts global
endpoints because the new session ID is not known yet. Explicit
`RemoveMcpEndpoint(Connection, Endpoint)` cancels outstanding carriers and
invalidates aliases; stale/foreign handles return `Closed`. Connection close
retires all endpoints. These are routing/lifetime constraints; ACP carriers
contain no trustworthy per-request session authorization.
