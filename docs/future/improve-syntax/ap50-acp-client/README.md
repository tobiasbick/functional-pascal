# AP50: Complete ACP client

Status: In progress. Effort: large; seven separately reviewable work packages.
Completion is tracked only in the [central plan](../README.md#package-status).
Package branch: `codex/ap50-acp-client`, based on `main`.

## Goal

FPAS programs connect to ACP agents, manage conversations, exchange every
supported content type, and answer agent requests while work continues.
The client covers the pinned Rust SDK's stable protocol and its explicitly
enabled experimental client features. A text-only prompt wrapper does not
satisfy this package.

Implementation on the package branch is authorized. Commit, push and merge
still require the corresponding user instruction. None is recorded as delivered
by the existence of these planning documents.

## Current behavior and proposed contract

The package branch pins the ACP SDK and provides complete detached v1,
experimental and v2 protocol DTOs with native payload codecs and lossless raw
values. Native protocol-value conversion, nominal IDs and generated editor
declarations are available. Connection operations remain in AP50.3 onward.
[Std.Proc](../../../pascal/std/host/proc.md#interactive-processes) already has
owned processes, nonblocking binary pipes, launch cwd/environment and cleanup.
[Std.Tasks](../../../pascal/std/concurrency/task.md) supplies channels, task
groups and cancellation. File streams, UTF-8 and Base64 are also implemented.
These are prerequisites on `main`, rather than new missing language features.

Add the public client modules described in [interface.md](interface.md).
Use the Rust SDK for ACP/JSON-RPC, with native VM-owned connection state and
typed FPAS values. The driver continues servicing the connection while the
application decides how to answer an inbound request. It never invokes FPAS
closures on an arbitrary native thread.

Preserve the existing language, task ownership, diagnostics and old `Std.Proc`
contracts. Add only the process operations specified in AP50.2. Do not change
`Std.Json.Number` or route ACP values through its lossy `real` conversion.

## Protocol baseline and completeness

Pin `agent-client-protocol = "=3.5.0"` and its schema dependency
`agent-client-protocol-schema = "=1.12.0"`. The SDK's
[release manifest](https://github.com/agentclientprotocol/rust-sdk/blob/v3.5.0/Cargo.toml)
records that schema version. The corresponding
[schema release](https://github.com/agentclientprotocol/agent-client-protocol/tree/schema-v1.26.0/agent-client-protocol-schema)
contains the Rust schema crate at version 1.12.0. Schema release numbering,
Rust crate numbering and ACP wire protocol versions are different identifiers.

[protocol-coverage.md](protocol-coverage.md) is the method/feature inventory.
Every applicable pinned schema field and variant must be represented, including
nullable patches, metadata, errors and unknown open-enum values. Stable ACP v1
is the default. Draft v2 and all seven experimental SDK protocol features are
included as explicit runtime opt-ins, with distinct versioned types. Compiling
their Rust features does not advertise or activate them for an FPAS connection.

SDK upgrades are deliberate changes to the pin, mapping and owning tests.
`latest`, Rust trait names in old tutorials, or a successful dependency build
are not completeness criteria.

## Decisions

| ID | Question | User decision | Affected work packages |
| --- | --- | --- | --- |
| Q55 | Which ACP role and how much functionality? | Client only. FPAS connects to agents, with complete client support. Prepare the AP before implementation. | AP50.1 through AP50.7 |
| Q56 | Can one inbound request admit multiple host effects? | No. Atomically admit at most one host execution per Responder, independently of its single response right. | AP50.3, AP50.5 |
| Q57 | How are long-lived resources bounded? | Configure explicit session, terminal, MCP-route and aggregate terminal-output limits with fixed defaults; reserve capacity before publication or effects. | AP50.3 through AP50.6 |

The architecture, dependency pins and defaults below are the design selected
within Q55 through Q57. No new syntax or language semantics are introduced.

## Open decisions

None blocks the described implementation. Native
registration and source modules may be split further to keep files focused;
the public contract and coverage must remain the same. A dependency defect
that prevents the pinned contract is a concrete blocker to repair or bring
back with evidence, not permission to mark a feature complete through raw RPC.

## Excluded scope

- A production ACP Agent, proxy or conductor role.
- A model provider, LLM inference implementation or replacement MCP SDK.
- Agent installation, registry management, automatic updates or distribution.
- A built-in chat application or editor integration. Examples demonstrate the
  library and host callbacks without becoming a second product.
- Standardized remote HTTP/WebSocket behavior that the pinned protocol has not
  finalized. The message transport adapter supports application-supplied links;
  that is not a claim of remote-protocol conformance.
- New syntax, general FFI, a new task system or a global JSON redesign.
- An OS sandbox. The library makes access decisions explicit; it does not
  claim to constrain an agent subprocess's independent OS privileges.

## Dependencies and work packages

Existing infrastructure on `main` is sufficient to start AP50.1 and AP50.2.
Dependencies are explicit in each file. The package branch retains their
results for subsequent work.

- [x] [AP50.1: Lossless schema types and codec](01-schema-and-types.md)
- [x] [AP50.2: Process status and interactive launch](02-process-support.md)
- [ ] [AP50.3: Native connection driver and transports](03-connection-runtime.md)
- [ ] [AP50.4: Complete stable v1 client](04-v1-client.md)
- [ ] [AP50.5: Inbound requests and host adapters](05-host-requests.md)
- [ ] [AP50.6: Draft v2 and experimental client features](06-experimental-client.md)
- [ ] [AP50.7: Distribution, examples and whole-client acceptance](07-acceptance.md)

This is the only work-package checklist. AP50.1 and AP50.2 are independent;
AP50.3 needs AP50.1 and uses AP50.2 for its subprocess adapter. AP50.4 needs
AP50.3. AP50.5 needs AP50.2 through AP50.4. AP50.6 needs AP50.1, AP50.3 and
AP50.5. AP50.7 integrates all of them. Each implementation package ships its
own documentation and contract tests; AP50.7 does not defer those obligations.

## Acceptance

An ordinary FPAS program can connect to a scripted conforming peer, exercise
the complete [coverage inventory](protocol-coverage.md), answer requests during
streaming, cancel work, and close without leaking native resources. The same
public interface works with a separately implemented ACP agent. Experimental
support is tested enabled and disabled and is labeled as experimental.

Rejected inputs fail before publication or host effects. Interleaved sessions,
replay, slow interaction, queue limits, peer exit and VM failure have specified
outcomes. Completion requires the whole-client verification in AP50.7, not
only a happy-path conversation or a raw escape hatch for missing typed methods.

## Documentation and regression owners

- Contract: [interface.md](interface.md) and
  [protocol-coverage.md](protocol-coverage.md).
- Implementation: new `crates/fpas-acp/`, existing native Std registration,
  `crates/fpas-vm/src/vm/hosted/acp/`, and public `Std.Acp` modules.
- Process changes: existing `hosted/proc/`, native process catalog and handbook.
- Public verification: `tests/stdlib/acp/` and `crates/fpas-cli/tests/acp/`.
- SDK codec/transport verification: `crates/fpas-acp/tests/`.
- Current documentation with implementation: `docs/pascal/std/ai/acp/`, the
  Std index, process handbook and compact reference routing entry.
- Apply [shared constraints](../shared-constraints.md), existing
  [Std implementation touchpoints](../../../pascal/std/README.md#shared-implementation-touchpoints)
  and the [test runner handbook](../../../pascal/std/testing/test.md).

## Live interoperability

Use the separately maintained [Codex ACP adapter](https://github.com/agentclientprotocol/codex-acp)
for authorized live smoke tests, selecting `gpt-6-luna` explicitly. Launch the
published `@agentclientprotocol/codex-acp` package and pass model configuration
through `CODEX_CONFIG` or the advertised session configuration. Never fall back
silently to a more expensive model. Use existing authentication without copying
credentials into the repository or test output. Automated acceptance remains
deterministic and independent of account access.
