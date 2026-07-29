# Phase 1 editor contracts

**Status:** confirmed on 2026-07-29  
**Scope:** editor tooling only; no FPAS syntax or semantic change

The machine-readable companion is
[`editors/vscode/contracts/phase1.json`](../../../editors/vscode/contracts/phase1.json).
`npm run verify:contracts --prefix editors/vscode` checks that its protocol,
source-API, host, and fixture references remain valid.

## Protocol baseline

- LSP version: 3.17 stable features only
- Transport: standard input and standard output
- Log transport: standard error
- Position encoding: UTF-16 code units
- Document synchronization: open/close plus full-document changes
- Save notification: enabled without repeating the complete document text
- Capability rule: advertise a capability only after its handler is implemented

The initial server contract is deliberately small:

| Phase | Feature | LSP methods | Language-service query |
|---:|---|---|---|
| 4 | Lifecycle | `initialize`, `initialized`, `shutdown`, `exit` | `fpas_lsp::Backend` |
| 4 | Documents | `didOpen`, `didChange`, `didSave`, `didClose` | `SynchronizedDocuments` over `DocumentStore::apply_full_text` |
| 5 | Push diagnostics | `textDocument/publishDiagnostics` | `diagnostics_for_document` |
| 5 | Formatting | `textDocument/formatting` | `format_document` |
| 6 | Document symbols | `textDocument/documentSymbol` | `document_symbols` |
| 6 | Hover | `textDocument/hover` | `hover` |
| 6 | Definition | `textDocument/definition` | `definition` |
| 6 | Completion | `textDocument/completion` | `completion` |

Incremental synchronization, semantic tokens, workspace symbols, rename,
references, code actions, and proposed LSP 3.18 features are not part of this
contract.

## Rust transport selection

The selected transport is `tower-lsp-server`, using version `0.23.0` as the
verified Phase 1 baseline. Phase 4 adds and locks that exact dependency in
`fpas-lsp`.

| Criterion | `tower-lsp-server` 0.23.0 | `lsp-server` 0.10.0 |
|---|---|---|
| Maintenance | Active community fork; Rust 2024 and Rust 1.85 baseline | Maintained inside rust-analyzer; Rust 2024 |
| Protocol types | Bundled `ls-types`; stable 3.17 coverage documented | Transport-only; protocol types are a separate dependency |
| Execution model | Async Tower service; Tokio-backed by default | Synchronous crossbeam channels and a caller-owned dispatch loop |
| Stdio and lifecycle | `Server` and `LspService` own framing, state, and stdio serving | `Connection` owns framing and handshake; the application owns the loop |
| Cancellation | Pending `$/cancelRequest` handling is built into the service | Queue primitives exist, but the application must route and cancel work |
| Testing | The service can be exercised directly without a child process | In-memory connections are testable, but dispatch remains application code |
| Dependency impact | Larger: Tower, Tokio, `ls-types`, and async support | Smaller: crossbeam, Serde, JSON, and logging |

`tower-lsp-server` is selected because it removes lifecycle, dispatch, and
cancellation plumbing from this hobby project while still allowing transcript
and service-level tests. The larger dependency graph is acceptable for the
single native server executable.

`lsp-server` is rejected for this implementation because its intentionally
low-level API would make FPAS own more concurrency, dispatch, and cancellation
code. Its smaller dependency footprint does not offset that maintenance cost.

Primary evidence:

- [`tower-lsp-server` repository](https://github.com/tower-lsp-community/tower-lsp-server)
- [`tower-lsp-server` feature coverage](https://github.com/tower-lsp-community/tower-lsp-server/blob/main/FEATURES.md)
- [`tower-lsp-server` 0.23.0 manifest](https://github.com/tower-lsp-community/tower-lsp-server/blob/main/Cargo.toml)
- [`lsp-server` source](https://github.com/rust-lang/rust-analyzer/tree/master/lib/lsp-server)
- [`lsp_server` 0.10.0 API](https://rust-lang.github.io/rust-analyzer/lsp_server/index.html)

## FPAS authority map

Existing crates remain authoritative. The language-service queries named below
compose those APIs and provide editor-oriented caching and indexes.

| Editor feature | Current authority | Contract and remaining service work |
|---|---|---|
| Syntax highlighting | FPAS grammar, examples, and tests | Phase 2 TextMate grammar; no compiler logic in TypeScript |
| Parsing | `fpas_parser::parse_compilation_unit` and `ParseDiagnostic` | Phase 3 `DocumentSnapshot` stores the AST and diagnostics by source revision |
| Diagnostics | `fpas-parser`, `fpas-sema`, `fpas-diagnostics::Diagnostic` | Phase 3 `diagnostics_for_document` exposes merged parser/sema results; Phase 5 converts and publishes them |
| Formatting | `fpas_fmt::format_source` | Phase 3 `format_document` formats the unsaved snapshot and returns no result after parse failure; Phase 5 returns the LSP edit |
| Project discovery | `fpas_project::load_project` and `load_workspace` | Phase 3 `WorkspaceContext` loads metadata and parsed-source graphs overlay open buffers without writing sidecars |
| Unit interfaces | `fpas_sema::analyze_unit` and `fpas_unit::UnitInterface` | Phase 3 project analysis caches dependency interfaces by participating source revisions |
| Document symbols | `fpas-parser` declarations and source spans | Phase 3 `DocumentSymbols` and `WorkspaceSymbolIndex` provide the declaration foundation; Phase 6 adds the hierarchical LSP query |
| Hover | AST spans, `ExprTypeMap`, and unit interfaces | `hover` formats declaration/type information at a source position |
| Definition | Project graph, AST spans, and compiler name-resolution rules | `definition` needs a stable declaration/reference index in `fpas-language-service` |
| Completion | Parsed declarations, project visibility, and unit interfaces | `completion` needs a stable visible-symbol query |

The Phase 3 service now composes the public parser, diagnostic, formatter,
project/workspace, semantic metadata, and unit-interface APIs behind stable
document snapshots and a declaration index. A stable
source-position-to-definition index and visibility-aware completion query are
still Phase 6 work. Any focused semantic API they require must preserve current
compiler behavior. No language change is required.

## Phase 4 implementation

The native server now enforces this baseline. Its initialize result advertises
UTF-16 plus full-document open/close/change/save synchronization and no later
capability. `tower-lsp-server` owns initialization-state errors, cancellation,
JSON-RPC parameter validation, framing, shutdown, and exit semantics.
`fpas-lsp` owns file-only URI conversion, UTF-16/UTF-8 conversion, ordered
document-store access, and stderr-only operational logging.

The development extension uses `vscode-languageclient` 10.1.0. That client
library can speak newer protocol revisions, but FPAS requests and advertises
only the stable LSP 3.17 contract recorded here. The client launches an
explicit repository or packaged path and never resolves `fpas-lsp` through the
system `PATH`.

## Host contract

The final native package script recognizes these local build targets:

- `win32-x64`
- `win32-arm64`
- `linux-x64`
- `linux-arm64`
- `darwin-x64`
- `darwin-arm64`

It builds only the current host. There is no cross-compilation, target matrix,
CI build, or publication.

An unsupported local combination must fail before packaging with:

```text
Unsupported Functional Pascal VSIX host target: {platform}-{arch}. Build on Windows, Linux, or macOS using an x64 or arm64 host.
```

Remote SSH, WSL, and container extension hosts are outside this hobby-project
contract. When a native server would start remotely, activation must stop with:

```text
Functional Pascal language-server support is unavailable in remote extension hosts. Open the workspace in a local desktop editor; remote SSH, WSL, and container extension hosts are not supported by this hobby-project build.
```

The extension does not inspect or special-case the editor product name.

## Fixture contract

Fixtures live under
[`editors/vscode/test/fixtures/`](../../../editors/vscode/test/fixtures/).

| Fixture | Expected result | Coverage |
|---|---|---|
| `standalone/features.fpas` | valid | program, record, function, all comment forms, escaped string, Unicode text |
| `standalone/malformed_syntax.fpas` | `F1001` | malformed syntax and parser recovery |
| `standalone/unicode_identifier.fpas` | `F0012` | current ASCII-only identifier rule |
| `workspace/phase1.fpasworkspace` | valid | program project consuming an exported library unit |

Unicode is valid in FPAS string contents, but identifiers currently permit only
ASCII letters, digits, and `_`. The negative Unicode-identifier fixture records
that implemented rule; it does not propose a language change.
