# AP02: Diagnostic implementation map

Package: [AP02: Structured diagnostics](README.md)

Status: complete. The shared model carries code identity, authoritative source
paths, optional positions and producer-supplied expected/found through lexer,
parser, sema, compiler, project/build/linker, VM, CLI, runner, debugger and editors.

## Producer and consumer inventory

| Area | Current path | Behavior |
|---|---|---|
| Code identity and catalog | `fpas-diagnostics/src/{code.rs,codes.rs}` | FPnxxx display; unique allocations, phase boundaries and complete reference rows |
| Shared record and locations | `fpas-diagnostics/src/{diagnostic.rs,location.rs,span.rs}` | Optional spans and producer-supplied expected/found; synthetic point identity |
| Coordinate resolution | `fpas-diagnostics/src/source_range.rs` | One-based Unicode scalar coordinates, exclusive real ends, CR/LF/CRLF and invalid UTF-8 bounds |
| Output adapters | `fpas-diagnostics/src/{file_diagnostic.rs,render.rs,render/json.rs}` | Authoritative optional path; text and JSON preserve identity/message/hint |
| Lexer | `fpas-lexer/src/` | Byte-span producers, allocated FPnxxx codes, optional-span consumers |
| Parser | `fpas-parser/src/parser/{core.rs,decl/parameters.rs}` | Structured token expectations; FP2014 for comma/grouped parameters |
| Semantic analysis | `fpas-sema/src/check/decl/mod.rs` | Structured expected/found type names from the shared compatibility check |
| Compiler | `fpas-compiler/src/error.rs` | Existing records retain optional spans; compiler phase remains distinguishable |
| Project and workspace | `fpas-project/src/{source/,manifest.rs,loading/,unit_graph/,workspace/}` | Read/UTF-8/source failures retain records and authoritative file; validation failures are coded |
| Build | `fpas-build/src/{engine/error.rs,program_artifact/,source_snapshot.rs,distribution/}` | Parser/compiler/project records survive orchestration; native LinkError is retained |
| Linker | `fpas-linker/src/error.rs` | Each LinkError category has an explicit project/build-range code |
| CLI | `fpas-cli/src/{cli_input/,cli_output/,cli_check/,cli_build.rs,cli_run.rs,cli_test/}` | Four commands share text/JSON reporting; JSON stderr contains only records |
| Runner | `fpas-cli/src/{bin/fpas-runner.rs,cli_test/process/}` | Diagnostic mode crosses process boundaries; native runner uses FPAS_DIAGNOSTICS without consuming application arguments |
| Runtime | `fpas-vm/src/vm/{diagnostics.rs,hosted/proc.rs}` and `fpas-std/src/proc.rs` | Missing source stays absent; known points have no invented end; child stderr becomes program-output records in JSON mode |
| Debugger | `fpas-debug/src/{breakpoints/runtime_failure.rs,jsonl/encode_record/}` | FPnxxx filters and output; absent runtime coordinates remain null |
| Language service and LSP | `fpas-language-service/src/` and `fpas-lsp/src/{diagnostics/,semantic_tools/}` | Optional positions, source identity, Unicode-to-editor conversion and FPnxxx quick fixes |
| VS Code | `editors/vscode/src/workflow/diagnostics.ts` and its tests | Workflow parser and integration assertions use the confirmed code prefix |

Paths in this inventory are relative to `crates/` unless they start with
`editors/`.

## Record and stream rules

- Phase is derived from the code. FP4000–FP4099 is compiler; FP4100–FP4999
  is project/build/linker/CLI/runner. Internal invariants retain FP9xxx.
- Source identity is retained in the span. FileDiagnostic carries the path;
  the JSON `source` and `location` fields carry the file and coordinates.
- A byte span resolves against its matching UTF-8 text. Columns count scalars,
  not UTF-8 bytes or UTF-16 units; real ends are exclusive. A missing span is
  null. A runtime point has a start and a null end.
- Expected/found are supplied directly by token/type producers. Text is only
  rendered at boundaries; errors are not reconstructed from Display output.
- JSON Lines use stderr. Progress/test summaries are suppressed there. Program
  stdout and existing exit statuses are preserved. Child stderr is wrapped as
  `kind: program-output`, while RunCapture still captures its result normally.
- CLI arguments after `--` belong to the application. Native applications
  select diagnostics through `FPAS_DIAGNOSTICS=json` and keep all their arguments.

## Regression coverage

Catalog tests validate unique code allocations, phase ranges and reference
coverage. Shared-record tests cover UTF-8/scalar coordinates, missing spans,
points and structured fields. Project/build/linker tests preserve source identity
through dependencies. Real-process CLI tests cover JSON stream purity, child
stderr, worker capture/truncation and unchanged stdout/exit behavior.
Debugger, language-service, LSP and editor tests cover transport consumers.

The [diagnostics reference](../../../pascal/tools/diagnostics.md) is authoritative
for allocated codes and public record fields.
