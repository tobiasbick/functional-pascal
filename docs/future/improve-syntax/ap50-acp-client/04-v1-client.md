# AP50.4: Complete stable v1 client

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

[AP50.1](01-schema-and-types.md) and
[AP50.3](03-connection-runtime.md) are delivered on the selected branch.
No new language or parser decision is required.

## Contract and scope

Implement every stable outgoing method, notification, incoming update and
content family in [protocol-coverage.md](protocol-coverage.md). Use
`Std.Acp.Send/Notify` with typed request/notification variants; do not add a
parallel named wrapper for every method unless it removes caller-owned state.
Applications keep session IDs from responses and correlate sent requests by
their local RequestToken.

Initialization is explicit and occurs once per connection. Validate the
requested version and capability object before publication. During initialization
only protocol-valid authentication/elicitation-related incoming traffic may be
handled; ordinary session work requires successful initialization. A failed
initialization never leaves a partially Ready connection. Return peer info,
version, all capabilities and auth methods in the typed response/state snapshot.

Capability gates apply to omitted, null, false, empty and populated objects
according to their actual schema meanings. An empty object can advertise an
object capability; an empty elicitation object advertises neither form nor URL.
Session mode/config/prompt checks use the negotiated agent capabilities and
the session's advertised values, not hardcoded provider/model names.

Retain sufficient per-session state for capability checks, active v1 prompt
ownership and pending permission cancellation. Allow concurrent independent
sessions and ordinary unrelated requests. Reject a second active v1 prompt for
the same session with Busy before sending. Do not infer prompt completion from
an empty text chunk, tool completion or transport silence.

Install load/replay routing before publishing session/load. Deliver historical
updates before the load response in received order. Resume and load remain
different operations. Listing returns the peer's next cursor; the library does
not silently auto-page or hide skipped entries. Closing releases active-session
resources; deletion removes history under the agent's documented capability.

Expose every stable update as received rather than storing an unbounded
transcript. Preserve message IDs, tool-call update patches, plan replacement,
session config state, notices and compaction identities. An application may
reduce these events into its own bounded UI state; that is not connection state.

`session/cancel` sends the notification and settles pending permissions for
that session as cancelled. It continues receiving the prompt's actual final
response and late permitted updates. Request-local cancellation has its own
token, wire notification and completion. Simultaneous cancel/response must not
publish two terminal outcomes or cancel another session.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-acp/src/client/v1.rs`, `capabilities.rs`, `sessions.rs`, `cancellation.rs` | Stable state, guards, request publication and response settlement |
| Create/modify | `crates/fpas-vm/src/vm/hosted/acp/client.rs`, `values/` | FPAS typed Send/Notify and version/state conversion |
| Modify | `crates/fpas-sema/src/std_registry/loaded/acp/`, native symbol/catalog owners | Complete public stable operations and signatures |
| Create | `crates/fpas-acp/tests/v1/` | Peer-driven method, capability and state contracts |
| Create | `tests/stdlib/acp/client_test.fpas`, `content_test.fpas`, `cancellation_test.fpas` | Public client event handling and type use |
| Create | `docs/pascal/std/ai/acp/sessions.md`, `content.md` | Delivered session, update, content and cancellation rules |
| Modify/generate | `docs/pascal/std/ai/acp/connections.md`, `lib/api/Std/Acp.fpas`, `lib/api/Std/Acp.Types.fpas` | Current connection states and signatures |

Keep each public behavior's primary regression at its strongest relevant
boundary. Reuse AP50.1's wire fixtures and AP50.3's process/queue fixtures.

## Acceptance and verification

- Every stable outgoing method has an independently specified peer response,
  and every optional method is rejected locally when its capability is absent.
  Required methods remain callable without unrelated optional capabilities.
- Cover initialize/auth-required/authenticate/logout; supported/unsupported
  versions; null/omitted/empty-object capabilities; and advertised mode/config
  values. Agent-declared terminal authentication is routed to AP50.5 rather
  than sent as authenticate.
- Two interleaved sessions, list pagination, full new/load/resume setup,
  additional workspace roots and all three MCP descriptor transports retain
  their correct fields and ownership.
- A peer emits replay before the setup response. All replayed messages reach
  the application once, without being mistaken for a new prompt completion.
- Exercise text, image, audio, resource and resource-link content in both
  directions, plus diffs/terminal references in tools and arbitrary metadata.
- Every stable session update is observable. Unknown open enums remain unknown;
  missing/null/replacement patches and repeated compaction chunks retain order.
- A blocked permission request receives cancelled when its session is cancelled;
  an unrelated session continues. Request cancellation, local receive timeout,
  peer errors and terminal connection failures remain distinguishable.

Run owning codec/client/VM tests, the public FPAS ACP cases, API declaration
checks and the required Rust/library checks. No real provider credentials are
needed for deterministic acceptance.
