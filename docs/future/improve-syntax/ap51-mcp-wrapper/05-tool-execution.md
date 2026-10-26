# AP51.5: Tool registration and application examples

Package: [AP51](README.md). Status: Complete.

## Registration and execution

Complete `Mcp.RegisterTool` and `Acp.RegisterTool` against one catalog and
dispatch implementation. Reject duplicate names before publication.
Advertised schemas agree with executable definitions. Use official
`ServerHandler::{list_tools,get_tool,call_tool}`, SDK protocol validation and
the shared offline validators for registered input/output schemas.

Only explicitly registered tools are exposed. Validate arguments and apply
application policy before effects. Peer annotations and ACP tool-call status
updates grant no authority and do not invoke handlers.

Supply structured arguments, request/attachment identity and cancellation to
the handler. Preserve content and structured results. Distinguish invalid
arguments, denied work, tool failure, protocol failure, capacity, I/O,
cancellation and closed resources. Tool `isError` differs from JSON-RPC failure.

The cooperative serving operation invokes registered FPAS callbacks and owns
protocol polling/reply bookkeeping. If state belongs to another task, the
handler forwards a bounded request and awaits that owner's result. The owner
checks current context and application rules before applying changes.
An accepted result describes the actual outcome.

## Neutral application examples

Create an MCP-only program using `Mcp.RegisterTool` and a program using
`Acp.RegisterTool` with normal ACP setup. Include an existing-server attachment
path. Reuse the tool definitions and handlers across examples.

Use a small bounded key/value store with read and conditional-update tools.
Keep the store in its owning FPAS task. Reads report the current value and
revision. Updates check the expected revision and execution policy before
changing the value and incrementing an effect counter.

Use these examples and deterministic fixtures to demonstrate:

- Structured discovery, argument validation and results.
- Callbacks accessing current application state through explicit ownership.
- Allowed updates, denied updates and stale revisions.
- Invalidated application contexts, cancellation and delayed outcomes.
- Reusing one MCP server through ACP and independently after ACP detach.

Automatic schema derivation from arbitrary signatures, a general agent
framework and application-specific business rules are outside this package.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/catalog.rs`, `crates/fpas-mcp/src/server/` | Shared catalog, SDK dispatch and results |
| Modify | `crates/fpas-vm/src/vm/hosted/mcp/`, MCP/ACP registration owners | Callback serving and convenience delegation |
| Create | `tests/stdlib/mcp/state_test.fpas`, `tests/stdlib/mcp/acp_tools_test.fpas`, `tests/stdlib/mcp/borrowed_test.fpas` | Distinct independent and ACP registration behavior |
| Create | `examples/network/mcp-tools/`, `examples/network/acp-mcp-tools/`, `examples/network/mcp-tool-store/` | Independent serving, shared state handlers and existing-server attachment |
| Create/modify | `docs/pascal/std/ai/mcp/`, `docs/pascal/std/ai/acp/` | Registration, serving, state ownership and errors |

## Acceptance

Test listing/handler agreement, duplicate and unknown names, invalid arguments,
policy before effects, tool errors and dynamic results at the owning MCP
boundary. ACP tests cover delegation, advertisement and lifetime rather than
repeating all MCP codec cases.

Verify reads see current state, an accepted update changes it once and
denied/stale/cancelled queued calls leave it unchanged. Duplicate/late replies
cannot execute the handler again. Check context invalidation, callback capture
cleanup and normal/debugger serving. Live Luna acceptance follows in AP51.6.
