# CLI/DAP completion

Status: **Open**. IDE debugging is complete and independent of this work.

## Current boundary

Existing terminal configuration connects nvim-dap directly to unmodified
NetCoreDbg over stdio DAP. Normal source controls, variables and output are
available. [Integration setup](../../../integrations/neovim/README.md) describes
its limits. FPAS exposes no debug CLI command.

Use existing backend/client capabilities. Keep the managed assembly, matching
PDB/source, backend executable, cwd and program arguments explicit. Preserve
ordinary FPAS execution and the compiler's [debug model](../../architecture/debug-information.md).

## Remaining work

- [ ] Resolve target stdin. NetCoreDbg's selected DAP path provides neither an
  input-write request nor runInTerminal; the debugger REPL is not program input.
  Choose an input route before accepting interactive input programs.
- [ ] Define startup, individual-request and shutdown deadlines with cancellation.
  Do not time out an entire interactive session or legitimate breakpoint idle time.
- [ ] Establish ownership and cleanup of launch-owned targets after adapter EOF,
  crash or hang. Prove containment before deliberately provoking an adapter hang.
- [ ] Verify normal exit, user stop, failed launch, missing backend, target
  exception, unresponsive requests and client exit with an active session.
- [ ] Complete terminal-client acceptance for current/stopped values, frame
  changes, input/output and clean one-action session/client termination.
- [ ] Verify the selected CLI/DAP route on Linux and Windows, documenting concrete
  platform limits and external tool/license requirements.
- [ ] Update the terminal integration instructions and close the corresponding
  [debugger](../debug-gap.md) and
  [documentation](../documentation-gaps.md#d3-concrete-debuggingprofiling-walkthroughs) gaps.

## Rules

Keep DAP traffic separate from target output and diagnostic logs. Invalidate
frame/variable references on resume. Use backend expression syntax explicitly;
full Pascal expressions and arbitrary attach are outside this scope. Any required
input/ownership design must preserve source stops and reliable cleanup.

**Batch debugging remains blocked:** no one-shot snapshots/reports, command-file
runners, hidden workers, alternate MI/JSONL route or new execution engine.
No backend fork or extra supervisor is selected. Existing test drivers are
regression support, not a product batch interface.
