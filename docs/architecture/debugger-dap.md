# DAP frontend

The adapter implements the Microsoft Debug Adapter Protocol schema at commit
[`d2387e875a1ec4c2ed9964edea22617c81651cc0`](https://github.com/microsoft/debug-adapter-protocol/blob/d2387e875a1ec4c2ed9964edea22617c81651cc0/debugAdapterProtocol.json).
DTO construction and independent protocol tests target this revision. The schema
is licensed under MIT in its upstream repository.

`DapTransport` owns byte-counted Content-Length framing and a bounded output
queue. `DapServer` owns protocol sequencing, configuration, stop-scoped integer
handles and translation to the common worker host. The default launch closes
target stdin. Private-terminal launch keeps DAP traffic separate from terminal
input and output. The adapter-owned TUI controller retains the last published
frame independently of the target. [TUI debug requests](../pascal/tools/debugging.md#private-terminal-and-tui-control)

Only supported capabilities are advertised. Attach, restart, mutation, calls,
conditional/data/function breakpoints and logpoints are outside this baseline.

## Source files

<details>
<summary>Browse 14 source files</summary>

- `src/Fpas.Debugging/Dap/`
  - [DapArguments.cs](../../src/Fpas.Debugging/Dap/DapArguments.cs)
  - [DapServer.Events.cs](../../src/Fpas.Debugging/Dap/DapServer.Events.cs)
  - [DapServer.Execution.cs](../../src/Fpas.Debugging/Dap/DapServer.Execution.cs)
  - [DapServer.Inspection.cs](../../src/Fpas.Debugging/Dap/DapServer.Inspection.cs)
  - [DapServer.cs](../../src/Fpas.Debugging/Dap/DapServer.cs)
  - [DapTransport.cs](../../src/Fpas.Debugging/Dap/DapTransport.cs)
  - [StopHandles.cs](../../src/Fpas.Debugging/Dap/StopHandles.cs)
- `tests/Fpas.IntegrationTests/Debugging/`
  - [DapDebuggerTests.Gallery.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.Gallery.cs)
  - [DapDebuggerTests.Inspection.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.Inspection.cs)
  - [DapDebuggerTests.Lifetime.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.Lifetime.cs)
  - [DapDebuggerTests.PrivateTerminal.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.PrivateTerminal.cs)
  - [DapDebuggerTests.Transport.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.Transport.cs)
  - [DapDebuggerTests.cs](../../tests/Fpas.IntegrationTests/Debugging/DapDebuggerTests.cs)
  - [DapProbe.cs](../../tests/Fpas.IntegrationTests/Debugging/DapProbe.cs)

</details>
