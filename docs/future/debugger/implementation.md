# MCP debugger implementation checklist

All steps are **Planned**. The [contract](mcp-debug.md) owns command/tool behavior.
Implement in order; update each checklist with actual results and blockers.
Keep fixtures, traces and reports under `.temp-data/`, not in the general guide.

## MCP-01: Host and external backend

- [ ] Add the official ModelContextProtocol dependency and focused debugging
  library; preserve compiler/Core dependency boundaries.
- [ ] Bind `fpas debug --netcoredbg <executable>` with normal help/argument errors.
- [ ] Host MCP over stdio using the SDK; keep every log on stderr.
- [ ] Validate explicit backend selection without downloading/bundling tools.
- [ ] Verify initialization/tool discovery using an SDK MCP client over real pipes.
- [ ] Establish and test process ownership on Linux/Windows, including a child
  whose parent exits before sending a target PID. Stop on an ownership blocker.

Completion: MCP connects cleanly; invalid backend configuration fails clearly;
backend/child ownership is demonstrated without adding a debug engine.

## MCP-02: DAP client and session lifetime

- [ ] Add bounded DAP framing, request correlation and a continuous event reader.
  Keep production code independent of the bounded regression-test driver.
- [ ] Implement initialize/launch/configurationDone, entry stop, breakpoint binding,
  thread/source state and normal termination against the actual backend.
- [ ] Track sessionId/stopId and invalidate frame/variable handles on resume.
- [ ] Implement request deadlines, cancellation, MCP EOF/shutdown and cleanup of
  owned processes with bounded fallback. Do not expire an idle stopped session.
- [ ] Verify normal exit, explicit stop, failed launch and repeated stop; then
  contained adapter EOF/crash/hang and outstanding-request completion.

Completion: a real launched target is controllable and no owned processes remain
on tested shutdown/failure paths. Unsupported capabilities produce clear results.

## MCP-03: Agent tools and structured results

- [ ] Implement the tools and schemas in the contract, including state/error codes.
- [ ] Implement status waiting, paged frames/variables and bounded categorized
  output with explicit cursors/truncation.
- [ ] Verify concurrent status waiting with pause/stop and execution events.
- [ ] Verify real FPAS source stops, all three step kinds, updated locals, shared
  captures, generic values and real async continuation after suspension.
- [ ] Verify backend expression evaluation, exceptions, stale handles, invalid
  requests, output limits and cancellation without false side-effect guarantees.
- [ ] Confirm missing symbols/source and unsupported stdin are explained; do not
  introduce an input tool that the backend cannot support.

Completion: an SDK MCP client can debug actual FPAS programs entirely through the
advertised tools, without raw DAP or terminal transcript parsing by the agent.

## MCP-04: Agent-host acceptance and documentation

- [ ] Exercise the handbook's NetCoreDbg download, runtime, path and MCP server
  setup from a prepared Linux and Windows environment.
- [ ] Verify an actual MCP-capable agent host can discover the tools, launch an
  FPAS DLL, stop at a breakpoint, inspect values, step and stop cleanly.
- [ ] Verify host disconnect/restart with a live target and no remaining processes.
- [ ] Update the [debug guide](../../pascal/tools/debugging.md), CLI help/reference,
  agent reference and test instructions to describe implemented behavior.
- [ ] Close the [MCP debugger gaps](../debug-gap.md) using actual acceptance results;
  remove completed planning content and retain only open work.

Completion: the documented agent workflow works. IDE acceptance remains complete;
no human CLI frontend, Neovim workflow or custom debugger engine is required.
