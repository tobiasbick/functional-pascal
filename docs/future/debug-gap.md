# CLI/DAP debugger gaps

IDE debugging is complete; the [guide](../pascal/tools/debugging.md) covers the
Microsoft VS Code route. Compiler debug metadata and value views are implemented.

CLI/DAP remains open under the [completion plan](debugger/cli-dap.md):

| Gap | Current limit |
| --- | --- |
| FPAS debug command | No CLI debug command is supplied. |
| Program input | The NetCoreDbg DAP route lacks target stdin/runInTerminal support. |
| Deadlines | Initialization warnings and disconnect timeouts do not provide general request cancellation. |
| Adapter failure | Cleanup of a live target after adapter crash/hang is not established. |
| Client exit | Complete termination with an active session is not accepted; stop the session before closing the client. |
| Platform acceptance | The terminal route needs complete Linux/Windows acceptance. |

The [Neovim configuration](../../integrations/neovim/README.md) provides normal
DAP controls and a stopped-state scope view. It does not close these gaps.
Existing [backend tests](../../tests/Fpas.IntegrationTests/Debugging/NetCoreDbgTests.cs)
cover source controls, values, captures, async, exceptions and normal disconnect;
[test setup](../../tests/README.md#netcoredbg-integration-tests) configures the
external backend.

Backend evaluation may execute program code. Full Pascal expression evaluation
and arbitrary attach are outside the current CLI/DAP scope.
**Batch debugging is blocked**, including snapshots/reports, command-file runners
and hidden batch workers. No backend patches, MI bridge or JSONL frontend are selected.
