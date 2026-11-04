# MCP agent debugger gaps

IDE debugging and compiler debug metadata are complete; see the
[debugging guide](../pascal/tools/debugging.md). `fpas debug` is a planned agent
interface using the official C# MCP SDK and external NetCoreDbg.

| Open capability | Plan |
| --- | --- |
| MCP command/server | Bind fpas debug, host stdio MCP and discover typed tools through the official SDK. |
| External debugger setup | Require the user's separately downloaded NetCoreDbg executable; no bundling or automatic acquisition. |
| DAP session client | Launch/initialize the backend, correlate responses/events and track current session/stop state. |
| Agent controls and inspection | Source breakpoints, execution controls, threads, frames, variables, evaluation and categorized output through MCP tools. |
| Bounds and lifetime | Request deadlines, stale-handle errors, paging/truncation, cancellation and owned-process cleanup on every exit/failure path. |
| Agent-host acceptance | Actual MCP client/agent workflow on Linux and Windows with documented backend/runtime requirements. |

The [contract](debugger/mcp-debug.md) and
[implementation checklist](debugger/implementation.md) own this work. Existing
[NetCoreDbg tests](../../tests/Fpas.IntegrationTests/Debugging/NetCoreDbgTests.cs)
cover compiler/backend compatibility; they do not implement the MCP product.

Initial scope excludes target stdin, arbitrary attach, full Pascal expression
evaluation, HTTP/REST endpoints, raw DAP/JSONL frontends and one-shot batch reports.
The Neovim configuration is not a dependency or acceptance target for this command.
