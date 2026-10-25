# Protocol coverage inventory

Package: [AP50](README.md). This is the required implementation inventory, not
a completion checklist. The numbered work packages own delivery and tests.

## Pinned sources

- [SDK manifest and feature flags](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/src/agent-client-protocol/Cargo.toml).
- [Stable outgoing requests](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/src/agent-client-protocol/src/schema/client_to_agent/requests.rs).
- [Stable incoming requests](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/src/agent-client-protocol/src/schema/agent_to_client/requests.rs).
- [Stable incoming notifications](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/src/agent-client-protocol/src/schema/agent_to_client/notifications.rs).
- [V2 method implementations](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/src/agent-client-protocol/src/schema/v2_impls.rs).
- [Full v1 schema](https://github.com/agentclientprotocol/agent-client-protocol/tree/schema-v1.26.0/agent-client-protocol-schema/src/v1).
- [Full v2 schema](https://github.com/agentclientprotocol/agent-client-protocol/tree/schema-v1.26.0/agent-client-protocol-schema/src/v2).

Use these versioned sources when a live documentation page or tutorial differs.
No raw request escape hatch counts as implementation of a known typed method.

## Stable v1 requests sent by the client

Each row maps to a typed `ClientRequest` variant and corresponding
`AgentResponse`, with all request/response fields from the pinned schema.

| Method | Required behavior | Owner |
| --- | --- | --- |
| `initialize` | Single negotiation; implementation info, capabilities and auth methods; explicit unsupported-version outcome | AP50.4 |
| `authenticate` | Agent-driven authentication by advertised method ID; auth-required errors preserved | AP50.4 |
| `logout` | Available only with advertised logout capability | AP50.4 |
| `session/new` | Cwd, additional roots, MCP descriptors, modes/config and returned session identity | AP50.4 |
| `session/load` | Capability gate; consume history updates before final response; preserve message IDs | AP50.4 |
| `session/resume` | Capability gate; resume without v1 load replay | AP50.4 |
| `session/list` | Cwd filter, cursor pagination, session entries and next cursor | AP50.4 |
| `session/delete` | Capability gate; history deletion is separate from closing an active session | AP50.4 |
| `session/close` | Capability gate; cancel active work and release session state | AP50.4 |
| `session/prompt` | Every content block and metadata; actual stop reason; no automatic retry | AP50.4 |
| `session/set_mode` | Advertised modes only; valid while idle or processing | AP50.4 |
| `session/set_config_option` | Complete selectors, grouped choices, Boolean values, categories and returned full option state | AP50.4 |

MCP descriptors include stdio command/args/env and HTTP/SSE URL/headers under
agent capability gates. The agent connects to these servers. Forwarding their
configuration does not require FPAS to implement those MCP clients itself.

## Stable v1 requests handled by the client

| Method | Required behavior | Owner |
| --- | --- | --- |
| `session/request_permission` | Every advertised option kind; selected ID or cancelled outcome; no automatic approval | AP50.5 |
| `fs/read_text_file` | Absolute path, session, optional one-based line and limit; disk or application response | AP50.5 |
| `fs/write_text_file` | Complete text and session; create absent file; explicit application authorization | AP50.5 |
| `terminal/create` | Command/args/env/cwd, bounded retained output, immediate terminal identity | AP50.5 |
| `terminal/output` | Current retained output, truncation flag and optional exact exit status | AP50.5 |
| `terminal/wait_for_exit` | Deferred response without blocking connection dispatch | AP50.5 |
| `terminal/kill` | Stop the command; keep terminal identity and final output | AP50.5 |
| `terminal/release` | Terminate remaining work and release native ownership | AP50.5 |
| `elicitation/create` | Form and URL modes, session/request scope, tool association and every outcome | AP50.5 |

Terminal authentication is an additional client capability, distinct from
`terminal/*`. AP50.2 and AP50.5 supply its interactive process and reconnect
flow. Form submission remains reviewable and editable by the user. URL mode
exposes agent identity, message and destination host before host navigation.

## Stable notifications and data

| Direction | Notification | Owner |
| --- | --- | --- |
| Client to agent | `session/cancel`, extension notifications | AP50.4 |
| Agent to client | `session/update`, `elicitation/complete`, extension notifications | AP50.4, AP50.5 |
| Either direction | `$/cancel_request`; original request is settled exactly once | AP50.3, AP50.4, AP50.5 |

The stable `session/update` variants are `user_message_chunk`,
`agent_message_chunk`, `agent_thought_chunk`, `tool_call`, `tool_call_update`,
`plan`, `available_commands_update`, `current_mode_update`,
`config_option_update`, `session_info_update`, `usage_update`, `notice`,
`compaction_update` and `compaction_summary_chunk`.

Preserve all discriminators, IDs, optional fields and patch semantics.
Message chunks group by optional opaque message ID, never by guessed text.
Tool-call updates distinguish an omitted field from an explicit replacement.
Compaction updates are ID-addressed, retain receive order and support patches,
summary replacement/chunks and replay. Notices are live user-facing events,
not history entries or fatal errors. Advertise the corresponding capabilities
only when the application accepts these events.

Content includes text, images, audio, text/blob embedded resources and resource
links, including MIME types, URIs, annotations and metadata. Tool-call content
also includes diffs and terminal references. Represent all kinds/statuses,
locations, raw tool input/output, plan entries/priorities, slash-command input
descriptors, usage/cost fields and all stop reasons. A full-content binding
does not require the library to render or decode media for display.

Every reachable supporting schema type belongs to AP50.1. Use nominal ID
types, full u64 values, raw arbitrary JSON and tri-state fields as described in
[interface.md](interface.md). Error code/message/data survive the bridge.
Generic extension requests and notifications work in both directions; names
retain their leading underscore exactly once. Unknown standard requests get
method-not-found; unknown notifications are surfaced as raw diagnostic events
without making the connection unusable merely because a method is new.

## Experimental client inventory

Build the following features from SDK 3.5.0 individually. Each defaults off in
`ExperimentalOptions`; enabling one does not enable another on the wire.

| SDK feature | Complete client requirement | Owner |
| --- | --- | --- |
| `unstable_end_turn_token_usage` | Typed prompt-end token usage and related response fields | AP50.6 |
| `unstable_llm_providers` | `providers/list`, `providers/set`, `providers/disable`, provider configuration and errors | AP50.6 |
| `unstable_mcp_over_acp` | Bidirectional `mcp/message` request/response/notification routing, scoped endpoints and cancellation | AP50.6 |
| `unstable_plan_operations` | `plan_update` and `plan_removed`, IDs and schema-defined operations | AP50.6 |
| `unstable_session_fork` | Typed `session/fork`, setup configuration and new active session identity | AP50.6 |
| `unstable_session_system_prompt` | Client system-prompt capabilities, setup fields and complete inheritance/override semantics | AP50.6 |
| `unstable_subagents` | `subagent_update`, `session_message`, `session_message_chunk`, ownership/restricted-session rules and participant/message IDs | AP50.6 |

`process`, `stdio`, `schemars` and `wasm_js` are SDK implementation features,
not additional protocol promises. The hosted FPAS client uses the custom
process adapter and generic SDK transport support. JSON-schema derivation,
an ACP Agent running on FPAS process stdio, and browser WebAssembly are not
requirements of the client role. Do not enable SDK process spawning as a
second unmanaged process implementation.

## Draft v2

`unstable_protocol_v2` is not included in the SDK's `unstable` umbrella.
AP50.6 enables and verifies it separately. Users must explicitly select v2;
stable initialization must not advertise it merely because it was compiled.

| V2 operation | Requirement |
| --- | --- |
| `initialize` | Typed version selection and explicit lower-version outcome |
| `auth/login`, `auth/logout` | V2 auth contract and advertised authentication availability |
| `session/new`, `session/resume`, `session/list`, `session/delete`, `session/close` | V2 baseline/capability rules and resume replay; no invented v1 `session/load` |
| `session/prompt` | Acceptance response is separate from foreground completion and later updates |
| `session/set_config_option` | V2 config contract; no invented v1 `session/set_mode` |
| `session/cancel`, `$/cancel_request` | Distinct session and request cancellation |
| `session/request_permission`, `elicitation/create`, `elicitation/complete` | Versioned permissions and elicitation; preserve all outcomes |
| `session/update` | Complete v2 content, message/state/tool/plan/terminal updates and metadata patches |
| Extensions and seven applicable opt-ins above | V2 schema types, gates and responses rather than v1 serialization |

V2 removes v1's filesystem/terminal capability methods. Follow the pinned v2
execution/MCP contract instead of advertising v1 `fs/*` or `terminal/*` on a
v2 connection. The separate host adapters can serve application-provided tools;
`mcp/message` preserves the inner MCP JSON without claiming a complete MCP SDK.

## Coverage ownership

AP50.1 verifies every represented type/field at the codec boundary. AP50.4,
AP50.5 and AP50.6 verify observable method behavior and state transitions.
AP50.3 owns transport, queue, concurrency and cleanup failures. AP50.7 proves
their composition through the public FPAS interface and an independent peer.
Adding another layer must exercise a different risk rather than duplicate the
same table or compare an encoder only with its own decoder.
