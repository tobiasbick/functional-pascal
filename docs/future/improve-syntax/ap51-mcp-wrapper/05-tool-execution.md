# AP51.5: FPAS tool providers and execution

Package: [AP51](README.md). Status: Not started; depends on AP51.2–AP51.4.

## Contract

Expose an explicitly configured FPAS tool catalog through `tools/list` and
dispatch `tools/call` to application code. Names, descriptions and schemas agree
with actual execution. Use official `ServerHandler::{list_tools,get_tool,call_tool}`
and SDK schema validation instead of a parallel validation engine.

Only selected tools are exposed. Listing or an ACP tool-call status update
does not authorize execution. Validate at the owning boundary and apply the
application's policy before observable effects. Peer annotations grant no
authority.

Return typed results/content and distinguish tool failure from protocol,
validation, permission, cancellation and I/O errors. Preserve structured JSON
where supported by the chosen revision. Consume response rights once and never
automatically replay uncertain effects after disconnect. Cancellation stops
cooperative work without claiming to undo completed effects.

Include a deterministic tool and an effect-counting fixture. Resources, prompts,
MRTR, subscriptions, tasks and external MCP forwarding are outside this workflow.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/server/tools.rs` | SDK catalog/dispatch adapter |
| Modify | `crates/fpas-vm/src/vm/hosted/mcp/`, `crates/fpas-sema/src/std_registry/loaded/mcp/` | Tool API and execution-policy boundary |
| Create | `tests/stdlib/mcp/tools_test.fpas`, owning Rust tests | Catalog, arguments, policy, results/errors and cancellation |
| Create | `examples/network/acp-mcp-tools/`, `docs/pascal/std/ai/mcp/tools.md` | Runnable Codex/ACP tool example and contract |

## Acceptance

Test listing/dispatch agreement, unknown tools, invalid arguments, denial before
effects, tool errors, dynamic results and cancellation. Verify one effect under
late replies, route reopen and disconnect. Exercise normal/debugger hosts and GC
cleanup of application state. AP51.6 verifies the actual Codex invocation.
