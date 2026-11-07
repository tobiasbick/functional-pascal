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
frame independently of the target. [TUI debug requests](../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)

Only supported capabilities are advertised. Attach, restart, mutation, calls,
conditional/data/function breakpoints and logpoints are outside this baseline.
