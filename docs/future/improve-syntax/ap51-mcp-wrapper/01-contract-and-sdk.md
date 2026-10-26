# AP51.1: Server contract and Codex compatibility

Package: [AP51](README.md). Status: SDK feasibility evidence available; target
transport/version verification is pending at home. Product work remains paused.

## Deliverable

Specify the FPAS MCP server whose tools Codex calls while connected over ACP.
Record the tested Codex/ACP-adapter versions, selected route, MCP revision,
tool-state boundary and SDK configuration. The product has no MCP client role.
The package README owns decisions Q58–Q64.

## SDK baseline and evidence

The published official `rmcp` 3.7.0 is an available baseline. It is not yanked,
requires Rust 1.88 and identifies upstream revision
[`5d6a11f2ae022f513aec0f6e0f10f89a57891a7f`](https://github.com/modelcontextprotocol/rust-sdk/tree/5d6a11f2ae022f513aec0f6e0f10f89a57891a7f).
Its registry SHA-256 is
`ece64e19631fc38758181e637c15509a2c2709ee53bfde1e1d58f9dc991f919e`.
Choose an exact published pin after confirming its target-route support.

Existing probes under `.temp-data/ap51-research/` built that release with both
SDK roles, HTTP and extra features on Rust 1.99.1. They exercised same-SDK
discovery, MRTR, cancellation, state integrity and dynamic JSON. Preserve them
as feasibility evidence; they do not verify the reduced server-only feature
set, FPAS binding or Codex compatibility. Their feature scope is not AP51 scope.

Start the product feature audit with `default-features = false`, `server`
and `schemars`. Add `transport-async-rw` for a selected byte-stream route or
`transport-streamable-http-server` for HTTP as required. Product client,
HTTP-client and automatic MRTR features are unnecessary. Reuse existing
crypto/TLS policy if needed. No floating Git dependency or replacement engine.

## Target check at home

1. Inspect the actual Codex ACP adapter's capabilities and MCP descriptors.
   Record Codex and adapter behavior separately.
2. Verify MCP-over-ACP support. Otherwise test a suitable local HTTP or stdio
   descriptor. Prove a real request reaches the SDK server.
3. Establish where tool state lives. A launched stdio server is a separate
   process; it does not automatically access the running ACP host's memory.
4. Verify MCP revision/lifecycle on the chosen route. Do not force modern
   discovery or remove initialization based solely on the linked introduction.
   Use the official SDK's selected lifecycle and validation.
5. Define endpoint identity, route access policy and cancellation/response
   ownership when ACP sessions or endpoints close.

Live target investigation is deferred to home. Do not attempt live Codex
experiments in the cloud; local fixtures cannot establish this target contract.

## Server API and ownership

Proposed units: `Std.Mcp.Types` and `Std.Mcp.Server`. Expose create/attach,
receive, respond/reject and stop/close/release through VM-local handles.
FPAS code explicitly dispatches requests; it does not run on the SDK executor.
No outbound Connect/Send, client continuation or external-server proxy API.

| Contract | SDK owner | FPAS owner |
| --- | --- | --- |
| Values/codec | `rmcp::model`, schema and serialization | Type conversion/registration; AP51.2 |
| Requests/context | `ServerHandler`, `RequestContext<RoleServer>` | Bounded dispatch and one-use response rights; AP51.3 |
| Tool discovery/calls | `ServerHandler::{list_tools,get_tool,call_tool}` and validation | Catalog, arguments, policy and FPAS execution; AP51.5 |
| Cancellation/close | Request stop token and service close facilities | Cooperative work and ACP endpoint lifetime; AP51.3/AP51.4 |
| Chosen transport | Official server transport/service APIs | ACP routing and stream/listener ownership; AP51.4 |

Distinguish invalid arguments, denied work, tool errors, protocol failure, I/O,
capacity, cancellation and closed resources. ACP tool status updates are
observations, not tool execution requests or permission grants.

## Limits and acceptance

Retain useful SDK findings: fixed service queues/pending work need FPAS admission
bounds; server bodies and stream frames need size limits; cancellation/shutdown
must remain live when ordinary admission is full. Bound payloads, catalogs,
responders and handlers, and check GC retention of application state.

Choose concrete defaults with the selected server route. Outbound HTTP body
adapters, client response caches, external-server polling, client continuations
and task managers are not prerequisites.

Update the package README and align AP51.2–AP51.6 after the home target check.
AP51.1 stays open until the actual route/version and concrete server contract
are verified. SDK probes alone do not complete it. Check links, scope consistency
and `git diff --check`.
