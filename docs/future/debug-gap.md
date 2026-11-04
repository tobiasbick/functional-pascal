# MCP agent debugger acceptance

The stdio MCP debugger is implemented and tested on Windows and Linux with
NetCoreDbg 3.2.0-1092 and .NET 10. Native job/process-group ownership, early parent
exit, adapter failure, explicit stop and client EOF have executable coverage.
See the [debugging guide](../pascal/tools/debugging.md) and
[implemented contract](../architecture/mcp-debugging.md).

| Open acceptance | Required evidence |
| --- | --- |
| Agent-host setup | Register the documented stdio server in an actual MCP-capable agent host on Linux and Windows; discover tools, launch, inspect, step, stop and disconnect with a live target. SDK client tests do not establish host configuration. |

This chat cannot dynamically register the local MCP server as conversation
tools. Keep this acceptance open until a configured agent host performs the
documented sequence; SDK-client tests do not substitute for it.

Target stdin, arbitrary attach, a Pascal evaluator, editor integrations, HTTP/raw
DAP frontends and one-shot batch reports are outside the selected scope.
