# Agent debugging with MCP

Status: **Implemented; platform/agent-host acceptance remains open**.
`fpas debug` is an MCP server for agents, using the official
[C# MCP SDK](https://github.com/modelcontextprotocol/csharp-sdk) and unmodified
[NetCoreDbg](https://github.com/Samsung/netcoredbg) over internal DAP.
Linux SDK-client integration tests exercise the command and real FPAS targets.

```text
Agent MCP client -> fpas debug -> DAP client -> NetCoreDbg -> FPAS program
```

| Document | Purpose |
| --- | --- |
| [Command and tool contract](mcp-debug.md) | Dependencies, launch, tools, state, output and lifetime |
| [Implementation checklist](implementation.md) | Ordered, testable delivery steps |

The user downloads NetCoreDbg separately and supplies its executable path.
FPAS does not bundle or automatically acquire it. The MCP SDK is a normal NuGet
dependency. FPAS owns the protocol/process integration; NetCoreDbg owns debugging.

IDE debugging is complete and documented in the
[VS Code guide](../../pascal/tools/debugging.md). IDE/editor integrations,
Neovim, a custom debugger engine, HTTP/REST endpoints, raw DAP/JSONL frontends
and one-shot batch reports are outside this implementation.

Related: [agent setup](../../pascal/tools/debugging.md#agent-debugging-with-mcp),
[debugger gaps](../debug-gap.md), [CLI inventory](../cli-gap.md#debugger-and-agent-inspection).
