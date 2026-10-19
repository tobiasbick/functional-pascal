# Tools

Compiler and editor tooling for Functional Pascal projects.

| Tool | Description |
|------|-------------|
| [Shared diagnostics](diagnostics.md) | Codes, source positions and Rust text/JSON rendering |
| [Source queries](source-queries.md) | JSON references, definitions, checked expression types, and safe rename through `fpas query` |
| [Runtime failure reports](failure-dump.md) | Opt-in `fpas run --failure-dump` with bounded task/frame/value JSON and atomic new-file publication |
| [Formatter style](fmt-style.md) | Normative output rules for `fpas fmt` |
| [Documentation examples](documentation-examples.md) | FPAS extractor and automatic compilation/execution checks for Markdown examples |
| [Debugger](debugger.md) | Source debugging through batch runs, JSONL, DAP, and VS Code |
| [Batch debugging](debugger-batch.md) | One command, bounded task/frame/value snapshot and JSON result |
| [Batch tracepoints](debugger-tracepoints.md) | Bounded scalar global/field and local/parameter history with declaration lifetimes |
| [Seeded batch scheduling](debugger-scheduling.md) | Repeatable fair task rounds, virtual task time and controlled effects through `--schedule-seed` |
| [Debugger JSONL protocol](debugger-jsonl.md) | Versioned machine-facing debugger contract |
| [Debugger DAP contract](debugger-dap.md) | VS Code-compatible capabilities and request mapping |
| [Editor integration](editor-integration.md) | VS Code-compatible highlighting, diagnostics, formatting, and navigation |
| [CLI](../program-structure/cli.md) | `fpas`, `check`, `test`, `fmt` discovery |

## See also

- [`fpas-fmt`](../../../crates/fpas-fmt) — formatter implementation
- [`fpas-language-service`](../../../crates/fpas-language-service) — compiler-backed editor analysis
- [`fpas-lsp`](../../../crates/fpas-lsp) — Language Server Protocol transport
- [`editors/vscode`](../../../editors/vscode) — VS Code-compatible extension and packaging
- [Projects](../program-structure/projects.md) — how the CLI discovers `.fpasprj` files
