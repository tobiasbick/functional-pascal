# MCP debugger implementation checklist

The command and tools are implemented. Linux acceptance uses .NET 10 and
unmodified NetCoreDbg 3.2.0-1092 with the official MCP SDK client. Windows and
actual agent-host registration remain open. The [contract](mcp-debug.md) owns
behavior; fixtures and local results belong under `.temp-data/`.

## MCP-01: Host and external backend

- [x] Official ModelContextProtocol 2.3.1 and hosting dependencies in the focused
  Fpas.Debugging library, independent of compiler/Core.
- [x] Required explicit backend selection, CLI help and actionable startup errors;
  stdio MCP with logging on stderr and SDK-client discovery over real pipes.
- [x] Linux ownership established before backend execution using setsid and
  process groups; subreaper cleanup tested after a parent exits without a PID event.
- [x] Windows suspended creation and kill-on-close Job Object implemented.
- [ ] Run native Windows ownership/early-parent-exit acceptance.

## MCP-02: DAP client and session lifetime

- [x] Bounded framing, correlated concurrent responses, continuous event handling
  and explicit rejection of unsupported reverse requests.
- [x] Initialize/launch/configurationDone, entry stop, breakpoints and normal exit
  against actual FPAS targets through MCP.
- [x] Fresh session/stop IDs, invalidated handles, deterministic stale errors.
- [x] Request/startup/shutdown deadlines, cancellation and cleanup; stopped
  sessions have no idle timeout.
- [x] Normal exit, explicit/repeated stop, failed launch followed by relaunch,
  malformed/hung adapters, concurrent disposal and outstanding-request completion.
- [x] Observed Linux MCP server exits on stdin EOF and removes its running target.
- [x] Actual NetCoreDbg target verified inside the owned Linux process group;
  killing the adapter alone reports failure and reaps the live orphaned target.

## MCP-03: Agent tools and structured results

- [x] All twelve tools with SDK-generated schemas, typed structured results and
  stable tool-error codes; no raw DAP or terminal transcript parsing.
- [x] Status waiting concurrent with pause/stop; cancellation leaves status usable.
- [x] Paged frames/values, Unicode-safe bounded output and explicit expired cursors.
- [x] Real source stops, all step kinds, updated locals, mutable shared captures,
  generic values and continuation after actual async suspension.
- [x] Backend evaluation, exception details, invalid requests and stale references.
- [x] Missing/mismatched symbols and changed sources fail clearly; stdin scope
  documented without an unsupported input tool.
- [x] Packaged SDK includes debugger/MCP/hosting dependencies and remains runnable.

## MCP-04: Agent-host acceptance and documentation

- [x] Current debug guide, CLI/agent reference, architecture and test instructions
  describe implemented tools, setup and verification limits.
- [ ] Follow the NetCoreDbg setup from a prepared Windows environment and verify
  cleanup with stopped/running targets and adapter failure.
- [ ] Register in an actual MCP-capable agent host on Linux and Windows and
  verify discovery, launch, source controls, inspection and disconnect/restart.
- [ ] Close the remaining [acceptance gaps](../debug-gap.md) using actual results.

The SDK MCP client exercises the product command; it does not replace the real
agent-host acceptance. The managed cloud chat provides no dynamic local MCP
registration. IDE debugging remains complete and independent of these checks.
