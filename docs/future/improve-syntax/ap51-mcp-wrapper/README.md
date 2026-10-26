# AP51: MCP tools for Codex over ACP

Status: Planned. Effort: Medium. Product implementation is paused; the current
authorization covers plan maintenance only. Completion is tracked in the
[central plan](../README.md#package-status).

## Goal

An FPAS application connects to Codex through `Std.Acp` and exposes its own
tools through an MCP server built on the official Rust SDK `rmcp`. Codex
discovers a tool, calls it, and receives the result of an explicitly dispatched
FPAS function. Codex is the MCP client; FPAS supplies the MCP server.

Codex is the first target. The user schedules target-agent transport/version
checks and live end-to-end acceptance at home. Do not attempt that live
verification in the cloud or count local fixtures as Codex interoperability.

## Current behavior

- [Std.Acp](../../../pascal/std/ai/acp/README.md) can advertise MCP server
  descriptors during session setup. Stdio, HTTP and MCP-over-ACP have distinct
  capability and lifecycle requirements.
- [ACP MCP endpoints](../../../pascal/std/ai/acp/experimental.md#mcp-carrier-endpoints)
  route opaque messages; registration does not implement a tool server or
  execute a tool. There is no `Std.Mcp.Server` binding in the checkout.
- ACP tool-call updates report agent activity. Execution of FPAS tools requires
  actual MCP requests, not interpreting those updates as commands.
- Existing handles, cooperative waits and
  [Std.Server](../../../pascal/std/network/server.md) provide lifetime patterns.
  AP50 is delivered; AP51 has no unfinished AP prerequisite.

## Agreed architecture and scope

Use [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
and its `rmcp` crate. The SDK owns MCP values, serialization, request handling
and transport protocol behavior. FPAS supplies conversion, explicit dispatch,
resource ownership, bounded queues and ACP integration.

Product scope is the MCP server role. Outbound FPAS MCP clients, external-server
proxying and a public client API are excluded. Test peers may use client
libraries without introducing a product client role.

The first tool workflow covers `tools/list`, `tools/call`, typed results,
tool/protocol errors, explicit permission policy, cancellation and shutdown.
Resources, prompts, sampling, roots, tasks, MCP Apps, MRTR, subscriptions,
managed OAuth and generic remote hosting are outside this delivery. Advertise
only implemented capabilities.

ACP integration is required. Select a transport from actual Codex support:

| Route | Required verification |
| --- | --- |
| MCP-over-ACP | Codex and its ACP adapter support the carrier; SDK requests/replies preserve IDs and lifetime |
| Local Streamable HTTP | Codex accepts the descriptor and reaches an owned loopback server with access to the running FPAS application's state |
| Stdio server process | Codex launches the descriptor; the process boundary fits tool state and keeps protocol output separate |

Choose the smallest suitable route; do not require all alternatives. A stdio
child does not share the ACP-host application's memory. Define any required
state transfer explicitly rather than introducing an unplanned IPC/proxy system.

The [MCP introduction](https://modelcontextprotocol.io/docs/2026-07-28/getting-started/intro)
is a reference, not a mandate to support only that revision. Select the MCP
revision/lifecycle supported by Codex and the official SDK. Do not impose modern
discovery or legacy initialization without verifying the target.

## Decisions

| ID | Topic | Current decision or required check | Owner |
| --- | --- | --- | --- |
| Q58 | Role | FPAS server only; Codex is the client | AP51.1–AP51.6 |
| Q59 | Target/API/protocol | Codex first; proposed `Std.Mcp.Types` and `Std.Mcp.Server`; verify actual revision/lifecycle at home | AP51.1, AP51.2, AP51.6 |
| Q60 | Transport | Select a working ACP-advertised route from Codex capabilities at home; all alternatives are not mandatory | AP51.1, AP51.4 |
| Q61 | Initial features | Tool discovery/invocation, results/errors, permission policy, cancellation and cleanup; additional MCP features are separate work | AP51.2, AP51.3, AP51.5 |
| Q62 | Authority/auth | Explicit tool exposure and authorization before effects; define any access policy/credentials with the chosen local route; no managed OAuth | AP51.4, AP51.5 |
| Q63 | ACP integration | Required outcome; MCP-over-ACP is a candidate route rather than an assumed capability | AP51.1, AP51.4, AP51.6 |
| Q64 | Implementation | Official `rmcp` SDK with an FPAS server wrapper, following ACP conventions | AP51.1–AP51.6 |

The transport/version compatibility check remains open until verified at home.
No FPAS syntax change is expected; any concrete syntax change requires user
agreement. Exact signatures follow existing ACP conventions and the selected route.

## Work packages and dependencies

- [ ] [AP51.1: Server contract and Codex compatibility](01-contract-and-sdk.md)
- [ ] [AP51.2: Server protocol types and API registration](02-types-and-codecs.md)
- [ ] [AP51.3: Owned server runtime and dispatch](03-server-runtime.md)
- [ ] [AP51.4: ACP-advertised MCP transport](04-acp-transport.md)
- [ ] [AP51.5: FPAS tool providers and execution](05-tool-execution.md)
- [ ] [AP51.6: Verification and Codex acceptance at home](06-acceptance.md)

This is the only work-package checklist. AP51.1 settles the SDK/server boundary
and target route/version; AP51.2 registers its API; AP51.3 owns dispatch;
AP51.4 connects it to ACP; AP51.5 supplies tools; AP51.6 accepts the result.
This plan update does not authorize product implementation.

## Acceptance

At home, FPAS establishes its ACP session with Codex and advertises the selected
MCP server. Codex discovers and calls a deterministic FPAS tool, which executes
once under the configured policy and returns the correct result. Invalid/denied
calls, tool errors, cancellation and close follow the agreed contract without
duplicate effects or retained rights/resources. Reopening a session/endpoint
never revives an old responder. Unrelated ACP events continue to progress.

Local automated tests cover the server independently of live Codex. AP51 remains
incomplete until the actual Codex compatibility check and end-to-end acceptance
at home pass. Cloud fixtures and SDK feasibility probes cannot check that box.
Follow the [development process](../development-process.md) and
[shared constraints](../shared-constraints.md).
