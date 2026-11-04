# MCP agent debugger acceptance

The stdio MCP command, official C# SDK integration, NetCoreDbg session client,
agent tools, limits and cleanup are implemented. Linux tests use the official
MCP client against real FPAS programs and NetCoreDbg 3.2.0-1092 on .NET 10.
See the [debugging guide](../pascal/tools/debugging.md).

| Open acceptance | Required evidence |
| --- | --- |
| Windows process ownership | Run Job Object startup/cleanup, adapter failure and client-disconnect tests on Windows. Compilation alone does not verify native process behavior. |
| Agent-host setup | Register the documented stdio server in an actual MCP-capable agent host on Linux and Windows; discover tools, launch, inspect, step, stop and disconnect with a live target. SDK client tests do not establish host configuration. |

The [implementation checklist](debugger/implementation.md) tracks these checks;
the [contract](debugger/mcp-debug.md) defines implemented behavior. The cloud chat
cannot dynamically register this repository's local MCP server as conversation
tools; use the configured local agent host for that acceptance.

Target stdin, arbitrary attach, a Pascal evaluator, editor integrations, HTTP/raw
DAP frontends and one-shot batch reports are outside the selected scope.
