# AP50.6: Draft v2 and experimental client features

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

[AP50.1](01-schema-and-types.md), [AP50.3](03-connection-runtime.md) and
[AP50.5](05-host-requests.md) are delivered. Stable v1 behavior remains covered
by AP50.4. Use the exact SDK/schema pins in the package README.

## Contract and scope

Implement all experimental client entries in
[protocol-coverage.md](protocol-coverage.md#experimental-client-inventory) and
the complete [v2 inventory](protocol-coverage.md#draft-v2). This work is required
for AP50 completion. Experimental does not mean a typed method can be omitted.
It means its availability and evolving wire version are explicit to callers.

Build the seven individually named unstable protocol features and
`unstable_protocol_v2`. Expose independent ExperimentalOptions booleans for
token usage, providers, MCP-over-ACP, plan operations, fork, system prompt and
subagents. The requested protocol version is a separate option. Defaults are
stable v1 with every experimental option false.

Reject an experimental request/field/advertisement while its option is disabled
before sending anything. Enabling a local option does not establish peer support;
also enforce the feature's pinned capability/negotiation rules. Preserve
unsupported future incoming variants as unknown raw events, without treating
them as activated features or triggering host effects.

## Experimental v1 behavior

- Provider operations preserve full configuration, optional credentials and
  peer errors. No provider hostname/model allowlist or ordinary logging of
  credentials is added by FPAS.
- Token usage is available in the exact response/update fields, without
  narrowing u64 counters to FPAS real or silently dropping absent values.
- Plan operations preserve ID-addressed changes/removal and ordering. They
  coexist with stable whole-plan updates according to the pinned schema.
- Fork publishes a new active session identity. Inherited/overridden cwd,
  workspace roots, MCP configuration and client system prompt follow the
  schema's presence/null semantics, not copied guesses from session/new.
- Subagent updates preserve parent/child ownership, participants and message
  IDs. Restricted observed sessions are not silently promoted to sessions on
  which the client can prompt/configure/close. Validate the pinned operation
  restrictions even when the caller possesses their session IDs.

## MCP-over-ACP routing

Expose typed ACP `mcp/message` request, response and notification envelopes with
their exact target/scope data. Inner MCP messages remain validated RawJson,
since they are another protocol. This is a complete ACP transport binding,
not a promise that FPAS implements every independent MCP server tool.

Provide public connection/session-scoped endpoint registration and removal.
The application forwards endpoint messages through the production message
transport or supplies its own tools. Register routes before publishing setup
requests, including fork/resume replay. Retain routes for the lifetime required
by the selected SDK protocol, rather than accidentally unregistering them when
a temporary session builder/value is dropped. Connection close cancels and
releases all routes and pending messages.

Route requests and responses independently in both directions. A nested MCP
request must not hold ACP's ordered dispatch loop while waiting on the caller.
Cancellation and target isolation cross the ACP/MCP routing layer explicitly.
Never turn a remote tool declaration into automatic local process/file access.

## V2 behavior

Keep `Std.Acp.V2` data and operations nominally distinct from stable v1.
Initialization uses the SDK's typed version-selection outcome. If a peer
selects a supported lower version, return that outcome; do not run v1 messages
through v2 decoders or resend initialization/prompts invisibly.

V2 `session/prompt` completes its request when accepted. Foreground completion
arrives later through the session state/update contract. Preserve acceptance,
prompt/message identity, background activity and idle stop reasons separately.
Do not release session state merely because the prompt request returned.
Apply v2's actual admission/busy rules rather than importing v1's single active
prompt guard without checking its contract.

Implement v2 resume replay, extended content/message updates, tool-call patches,
plan variants, metadata patches, permission changes and terminal-output data.
Preserve required/null/omitted distinctions and unknown open variants. The
whole current schema is the contract, including variants absent from the short
SDK example. V2 auth uses auth/login and auth/logout. V2 does not accept v1
session/load, session/set_mode, fs/* or terminal/* just because those features
exist in another FPAS module. Follow the pinned v2 execution/MCP declarations.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `crates/fpas-acp/Cargo.toml`, workspace Cargo metadata | Explicitly compiled protocol features and schema pin |
| Create | `crates/fpas-acp/src/client/v2.rs`, `experimental/` | V2 lifecycle and all seven experimental feature guards |
| Create | `crates/fpas-acp/src/mcp/` | Bidirectional scoped routing and retained route ownership |
| Create/modify | `crates/fpas-vm/src/vm/hosted/acp/experimental.rs`, `v2.rs`, `values/` | Public versioned operation/event bridge |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/acp/experimental.rs`, `v2.rs`, native names/catalog | Versioned public types and operations |
| Create | `crates/fpas-acp/tests/experimental/`, `tests/v2/` | Peer-driven feature, version and nested-routing contracts |
| Create | `tests/stdlib/acp/experimental_test.fpas`, `v2_test.fpas` | Ordinary FPAS callers and version/type separation |
| Create | `docs/pascal/std/ai/acp/experimental.md`, `v2.md` | Delivered opt-in contracts, limitations and version distinction |
| Generate | `lib/api/Std/Acp.Experimental.fpas`, `lib/api/Std/Acp.V2.fpas` | Accurate discoverable declarations |

Stable v1 is still shipped and tested with experimental options off. Keep the
implementation split by feature; a single conditional module containing every
experimental schema and state machine is not the intended owner.

## Acceptance and verification

- Every feature works against independently specified wire messages when
  enabled, and fails before publication/effects when disabled. Test capability
  combinations, not only an all-features-on peer.
- Prove fork identity/configuration and system-prompt missing/null/override
  behavior; provider errors and credential redaction; plan removal/order;
  subagent restricted operations; exact end-turn usage.
- Bidirectional nested MCP messages, session/global route isolation, setup-time
  replay, cancellation and route lifetime all work under slow responses.
- A v2 peer accepts a prompt, emits further work and later becomes idle. The
  client reports these as different events and continues background updates.
- V2 resume replay and all v2 update/permission/content families retain their
  field semantics. Cross-version requests/responses are rejected locally.
- A v2 initialization selecting v1 returns its typed lower-version outcome.
  No duplicate initialization, prompt or host effect is observed by the peer.

Run owning v2/experimental/MCP tests, stable v1 regressions, public FPAS cases,
API generation and required Rust/library checks. Label unsupported upstream
defects precisely; do not conceal them behind a permanently raw-only method.
