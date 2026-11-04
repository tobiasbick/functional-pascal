# Debugging status

IDE debugging is complete. Official VS Code with Microsoft's C# extension uses
FPAS assemblies, portable PDBs and standard CLR debug metadata. Setup and supported
value views belong in the [debugging guide](../../pascal/tools/debugging.md);
implementation details belong in [debug information](../../architecture/debug-information.md).

## Open: CLI/DAP

[CLI/DAP completion](cli-dap.md) covers program input, request deadlines,
adapter failure recovery, target ownership and terminal-client acceptance.
NetCoreDbg is the external backend. The
[Neovim configuration](../../../integrations/neovim/README.md) is an incomplete
terminal integration, not a completed FPAS CLI debugger.

**Batch debugging is blocked.** No one-shot snapshots/reports, command-file
runners, hidden batch workers, MI bridge or JSONL session frontend are selected.
Ordinary debugger evaluation may execute getters, display code and methods.

Related inventories: [debugger gaps](../debug-gap.md),
[CLI gaps](../cli-gap.md#debugger-and-agent-inspection),
[documentation gaps](../documentation-gaps.md#d3-concrete-debuggingprofiling-walkthroughs).
