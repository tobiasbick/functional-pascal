# AP02 implementation audit

Delivery: local implementation on `codex/syntax-changes-2`.
The six work packages are implemented together because the shared record,
transport and its consumers need to remain consistent. Delivery completion is
tracked in the [package checklist](README.md#work-packages); integration follows
the [development process](../development-process.md).

## Scope and reuse

This is a diagnostics/CLI/tooling change. Accepted FPAS syntax and semantics
remain the same. FP2014 recognizes two already-invalid parameter spellings and
recovers at the header boundary. Later block, import and mutation changes remain
with their owning packages.

Validation also exposed a pre-existing non-progress loop in record initializer
recovery, reached when folder discovery loaded an older scratch project.
The minimal input `record then` repeatedly emitted errors at the same token.
A progress check now returns to the enclosing parser at that boundary. The
three-second CLI regression failed before the fix and passed afterward;
parser tests cover literal/update boundaries and valid field initializers.
Scratch files were retained, and temporary instrumentation was removed.

JSON-mode test workers also retain child-process stderr through their bounded
output buffer. A real-process regression reproduced the missing records before
the fix and passed afterward for one and two workers. In-process tests cover
successful runs with one and two jobs; runtime failures retain their exit code
and JSON report on stdout.

Manifest failures now retain the authoritative project/workspace path, while
dependency and source failures keep their own paths. Missing precise positions
remain null. Program-stderr capture has a separate 8 MiB budget in the worker
protocol; overflow emits one complete program-output truncation event and leaves
the VM outcome and stdout report unchanged. Regression tests cover passing,
assertion-failing and runtime-failing tests with one and two workers. Both review
findings were reproduced by failing tests before their corrections.

The diagnostic transport from the reference branch through `c5b6c1a7` was reused
without its later syntax migrations or its superseded future plan. The shared
model was extended rather than introducing another diagnostic system. The
user's confirmed Q01 numbering supersedes the reference branch's numbering.

## Producer and consumer inventory

| Area | Existing path | Delivered behavior |
|---|---|---|
| Code identity and catalog | `fpas-diagnostics/src/{code.rs,codes.rs}` | FPnxxx display; unique allocations, phase boundaries and complete reference rows |
| Shared record and locations | `fpas-diagnostics/src/{diagnostic.rs,location.rs,span.rs}` | Optional spans and producer-supplied expected/found; synthetic point identity |
| Coordinate resolution | `fpas-diagnostics/src/source_range.rs` | One-based Unicode scalar coordinates, exclusive real ends, CR/LF/CRLF and invalid UTF-8 bounds |
| Output adapters | `fpas-diagnostics/src/{file_diagnostic.rs,render.rs,render/json.rs}` | Authoritative optional path; text and JSON preserve identity/message/hint |
| Lexer | `fpas-lexer/src/` | Existing byte-span producers, renumbered catalog, optional-span consumers |
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
`editors/`. Detailed changed paths follow below.

## Numbering

| Producer | Previous identity | Delivered identity |
|---|---|---|
| Lexer | F0001–F0013, with gaps | FP1001–FP1013, same gaps |
| Parser | F1001–F1013 | FP2001–FP2013; new FP2014 |
| Semantic analysis | F2001–F2019 | FP3001–FP3019 |
| Compiler | F3001–F3007 | FP4001–FP4007 |
| Project/build/linker/CLI/runner from reference branch | F5001–F5042 | FP4101–FP4142 |
| Runtime | F4001–F4025, with reserved F4017 | FP5001–FP5025, with reserved FP5017 |
| Internal invariants | F9001–F9003 | FP9001–FP9003 |

AP02 delivered 120 allocated codes, each with one cause/wrong/corrected row in
the [diagnostics reference](../../../pascal/tools/diagnostics.md). Later packages
extend that catalog and reference; the counts and verification below describe
AP02 delivery. Range membership does not allocate a code.

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

## Structural changes

- Move CLI output into `cli_output/mod.rs`, with focused `diagnostics.rs`,
  `failure.rs` and `program_output.rs`; move diagnostic CLI tests into their own directory.
- Separate project source I/O/errors, manifest errors, and unknown-unit hints.
- Add focused build error transport and its tests under `engine/error/`.
- Extract formal parameters from routine declarations into `decl/parameters.rs`.
- Extract allocation/reference tests into `fpas-diagnostics/src/codes/tests.rs`.
- Extract debugger event encoding into `jsonl/encode_record/events.rs`.
- Place the VM's program-stderr configuration with hosted process handling.

Existing large test files whose only extra changes are code identities are
kept in their current theme; unrelated restructuring is outside AP02.

## Verification

| Check | Result |
|---|---|
| `cargo fmt` and `cargo fmt --check` | Passed |
| `cargo build` | Passed |
| `cargo test --workspace` | 3363 passed, no failures; includes parser recovery, structured records, project/build transport, LSP/debugger consumers and real CLI processes |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | Passed |
| `fpas test tests/suite.fpasprj --report json` | 459 total: 458 passed, one skipped, no failures |
| `fpas fmt --check tests/manual/assert_fail_demo.fpas` | Passed for the changed FPAS source |
| `fpas fmt --check --list lib apps examples tests` | Passed after canonical formatting of the eight sources listed below |
| VS Code `compile.mjs`, `verify-contracts.mjs`, `run-tests.mjs` | Passed, including extension-host diagnostics, workflows and runtime-code filters |
| Diagnostic catalog/reference tests | 120 unique allocations, each documented once with cause/wrong/corrected columns |
| Executed reference samples | Wrong/corrected pairs for FP1007, FP1011, FP2001, FP2012, FP2013, FP2014, FP3005, FP3006 and FP3009 produce the expected code/pass check |
| Relative Markdown links and `git diff --check` | Passed |
| Added-content privacy and temporary-instrumentation scan | No findings |

The formatting cleanup applies canonical formatter output to these sources:

- `examples/math/mandelbrot/mandelbrot_app.fpas`
- `examples/network/tcp_parallel_echo_server.fpas`
- `examples/pascal/tui/notes-headless/notes_headless_benchmark.fpas`
- `lib/Std/Json/Fields.fpas`
- `lib/Std/Tui/Runtime/Application/ConsoleInput.fpas`
- `tests/stdlib/fs/fs_create_dir_all_test.fpas`
- `tests/stdlib/json/json_fields_typed_access_test.fpas`
- `tests/stdlib/toml/toml_fields_typed_access_test.fpas`

The formatter itself is unchanged. The existing formatter round-trip test binary
passes all four tests for the parser corpus and the examples, tests and apps
trees. Real literal normalization preserves the exact binary64 integer bounds
in `Std.Json.Fields`. The three formatted regression tests and the TUI host
signature test also pass with `--std-lib lib`, using the current repository
standard-library sources.

## Changed paths

- `Cargo.lock`
- `crates/fpas-build/src/distribution/mod.rs`
- `crates/fpas-build/src/engine.rs`
- `crates/fpas-build/src/engine/backend.rs`
- `crates/fpas-build/src/engine/error.rs`
- `crates/fpas-build/src/engine/error/tests.rs`
- `crates/fpas-build/src/lib.rs`
- `crates/fpas-build/src/program_artifact/atomic.rs`
- `crates/fpas-build/src/program_artifact/mod.rs`
- `crates/fpas-build/src/program_artifact/source.rs`
- `crates/fpas-build/src/program_artifact/tests.rs`
- `crates/fpas-build/src/source_snapshot.rs`
- `crates/fpas-build/tests/diagnostics.rs`
- `crates/fpas-build/tests/incremental.rs`
- `crates/fpas-bytecode/src/intrinsic/execution.rs`
- `crates/fpas-cli/src/bin/fpas-runner.rs`
- `crates/fpas-cli/src/cli_build.rs`
- `crates/fpas-cli/src/cli_check/directory.rs`
- `crates/fpas-cli/src/cli_check/mod.rs`
- `crates/fpas-cli/src/cli_debug.rs`
- `crates/fpas-cli/src/cli_fmt/paths.rs`
- `crates/fpas-cli/src/cli_input/discovery.rs`
- `crates/fpas-cli/src/cli_input/help.rs`
- `crates/fpas-cli/src/cli_input/mod.rs`
- `crates/fpas-cli/src/cli_input/options.rs`
- `crates/fpas-cli/src/cli_input/types.rs`
- `crates/fpas-cli/src/cli_output/diagnostics.rs`
- `crates/fpas-cli/src/cli_output/failure.rs`
- `crates/fpas-cli/src/cli_output/mod.rs`
- `crates/fpas-cli/src/cli_output/program_output.rs`
- `crates/fpas-cli/src/cli_run.rs`
- `crates/fpas-cli/src/cli_test/discover.rs`
- `crates/fpas-cli/src/cli_test/expect_stdout.rs`
- `crates/fpas-cli/src/cli_test/image/mod.rs`
- `crates/fpas-cli/src/cli_test/link.rs`
- `crates/fpas-cli/src/cli_test/log.rs`
- `crates/fpas-cli/src/cli_test/mod.rs`
- `crates/fpas-cli/src/cli_test/parallel.rs`
- `crates/fpas-cli/src/cli_test/process/mod.rs`
- `crates/fpas-cli/src/cli_test/process/output.rs`
- `crates/fpas-cli/src/cli_test/process/output/tests.rs`
- `crates/fpas-cli/src/cli_test/process/tests.rs`
- `crates/fpas-cli/src/cli_test/process/worker.rs`
- `crates/fpas-cli/src/cli_test/run/hook_exec.rs`
- `crates/fpas-cli/src/cli_test/run/load.rs`
- `crates/fpas-cli/src/cli_test/run/mod.rs`
- `crates/fpas-cli/src/cli_test/run/program.rs`
- `crates/fpas-cli/src/cli_test/runner.rs`
- `crates/fpas-cli/src/cli_test/tests/captured_output.rs`
- `crates/fpas-cli/src/cli_test/tests/discovery.rs`
- `crates/fpas-cli/src/cli_test/tests/golden.rs`
- `crates/fpas-cli/src/cli_test/tests/reporting.rs`
- `crates/fpas-cli/src/cli_test/tests/run_timeout.rs`
- `crates/fpas-cli/src/cli_test/tests/skip.rs`
- `crates/fpas-cli/src/cli_test/tests/validation.rs`
- `crates/fpas-cli/src/main.rs`
- `crates/fpas-cli/src/main_tests/diagnostics/json.rs`
- `crates/fpas-cli/src/main_tests/diagnostics/mod.rs`
- `crates/fpas-cli/src/main_tests/input/mod.rs`
- `crates/fpas-cli/src/main_tests/projects/build.rs`
- `crates/fpas-cli/src/main_tests/projects/enum_variants.rs`
- `crates/fpas-cli/src/main_tests/projects/errors.rs`
- `crates/fpas-cli/src/main_tests/projects/unwrap_diagnostics.rs`
- `crates/fpas-cli/src/main_tests/projects/warnings.rs`
- `crates/fpas-cli/src/main_tests/test_project.rs`
- `crates/fpas-cli/src/main_tests/test_project/exact_selection.rs`
- `crates/fpas-cli/src/main_tests/test_runner.rs`
- `crates/fpas-cli/src/main_tests/test_suite_negative.rs`
- `crates/fpas-cli/src/project/tests/loading/source_files.rs`
- `crates/fpas-cli/src/project/tests/support.rs`
- `crates/fpas-cli/src/project_build.rs`
- `crates/fpas-cli/src/standard_library.rs`
- `crates/fpas-cli/tests/json_streams.rs`
- `crates/fpas-cli/tests/json_streams/test_workers.rs`
- `crates/fpas-compiler/src/error.rs`
- `crates/fpas-compiler/src/tests/diagnostics.rs`
- `crates/fpas-debug/src/breakpoints/runtime_failure.rs`
- `crates/fpas-debug/src/evaluation/parse.rs`
- `crates/fpas-debug/src/jsonl/encode_record.rs`
- `crates/fpas-debug/src/jsonl/encode_record/events.rs`
- `crates/fpas-debug/src/jsonl/encode_record/tests.rs`
- `crates/fpas-debug/src/jsonl/parse/args.rs`
- `crates/fpas-debug/tests/dap_runtime_failure_filters.rs`
- `crates/fpas-debug/tests/record_replay.rs`
- `crates/fpas-debug/tests/runtime_failure_filters.rs`
- `crates/fpas-debug/tests/task_lifecycle.rs`
- `crates/fpas-diagnostics/Cargo.toml`
- `crates/fpas-diagnostics/src/code.rs`
- `crates/fpas-diagnostics/src/codes.rs`
- `crates/fpas-diagnostics/src/codes/tests.rs`
- `crates/fpas-diagnostics/src/diagnostic.rs`
- `crates/fpas-diagnostics/src/file_diagnostic.rs`
- `crates/fpas-diagnostics/src/lib.rs`
- `crates/fpas-diagnostics/src/render.rs`
- `crates/fpas-diagnostics/src/render/json.rs`
- `crates/fpas-diagnostics/src/source_range.rs`
- `crates/fpas-diagnostics/src/span.rs`
- `crates/fpas-diagnostics/tests/rendering.rs`
- `crates/fpas-diagnostics/tests/structured_output.rs`
- `crates/fpas-diagnostics/tests/value_invariants.rs`
- `crates/fpas-language-service/src/analysis/project.rs`
- `crates/fpas-language-service/src/semantic_tools/code_actions.rs`
- `crates/fpas-language-service/src/semantic_tools/mod.rs`
- `crates/fpas-language-service/src/workspace/context.rs`
- `crates/fpas-language-service/src/workspace/discovery.rs`
- `crates/fpas-language-service/src/workspace/standard_library.rs`
- `crates/fpas-language-service/tests/semantic_tools.rs`
- `crates/fpas-lexer/src/tests/comments.rs`
- `crates/fpas-lexer/src/tests/errors/unknown_tokens.rs`
- `crates/fpas-lexer/src/tests/identifiers.rs`
- `crates/fpas-lexer/src/tests/source_id.rs`
- `crates/fpas-linker/Cargo.toml`
- `crates/fpas-linker/src/error.rs`
- `crates/fpas-lsp/src/diagnostics/convert.rs`
- `crates/fpas-lsp/src/semantic_tools/code_actions.rs`
- `crates/fpas-lsp/tests/diagnostics.rs`
- `crates/fpas-lsp/tests/semantic_tools.rs`
- `crates/fpas-parser/src/parser/core.rs`
- `crates/fpas-parser/src/parser/decl/mod.rs`
- `crates/fpas-parser/src/parser/decl/parameters.rs`
- `crates/fpas-parser/src/parser/decl/routines.rs`
- `crates/fpas-parser/src/parser/expr/primary.rs`
- `crates/fpas-parser/src/tests/errors/diagnostics.rs`
- `crates/fpas-parser/src/tests/errors/mod.rs`
- `crates/fpas-parser/src/tests/errors/parameters.rs`
- `crates/fpas-parser/src/tests/errors/recovery.rs`
- `crates/fpas-parser/src/tests/errors/synthetic_eof.rs`
- `crates/fpas-project/src/dependencies.rs`
- `crates/fpas-project/src/lib.rs`
- `crates/fpas-project/src/loading/exports.rs`
- `crates/fpas-project/src/loading/mod.rs`
- `crates/fpas-project/src/loading/own.rs`
- `crates/fpas-project/src/loading/parse_cache.rs`
- `crates/fpas-project/src/manifest.rs`
- `crates/fpas-project/src/model.rs`
- `crates/fpas-project/src/paths.rs`
- `crates/fpas-project/src/source.rs`
- `crates/fpas-project/src/source/error.rs`
- `crates/fpas-project/src/source/read.rs`
- `crates/fpas-project/src/standard_library.rs`
- `crates/fpas-project/src/standard_library/tests.rs`
- `crates/fpas-project/src/test_manifest.rs`
- `crates/fpas-project/src/test_sources.rs`
- `crates/fpas-project/src/unit_graph/mod.rs`
- `crates/fpas-project/src/unit_graph/model.rs`
- `crates/fpas-project/src/unit_graph/order.rs`
- `crates/fpas-project/src/unit_graph/parsed.rs`
- `crates/fpas-project/src/unit_graph/program.rs`
- `crates/fpas-project/src/unit_graph/resolve.rs`
- `crates/fpas-project/src/unit_graph/unknown_unit.rs`
- `crates/fpas-project/src/workspace/discover.rs`
- `crates/fpas-project/src/workspace/loading.rs`
- `crates/fpas-project/src/workspace/resolve.rs`
- `crates/fpas-project/src/workspace/test_discover.rs`
- `crates/fpas-project/tests/diagnostics.rs`
- `crates/fpas-project/tests/loading.rs`
- `crates/fpas-project/tests/loading_edges.rs`
- `crates/fpas-project/tests/parsed_unit_graph.rs`
- `crates/fpas-project/tests/project_integrity.rs`
- `crates/fpas-project/tests/unit_graph.rs`
- `crates/fpas-sema/src/check/decl/mod.rs`
- `crates/fpas-sema/src/tests/integration/diagnostics.rs`
- `crates/fpas-std/src/error.rs`
- `crates/fpas-std/src/lib.rs`
- `crates/fpas-std/src/proc.rs`
- `crates/fpas-vm/src/vm/debug/recording/effects.rs`
- `crates/fpas-vm/src/vm/debug/session/recording.rs`
- `crates/fpas-vm/src/vm/debug/tests/recording.rs`
- `crates/fpas-vm/src/vm/diagnostics.rs`
- `crates/fpas-vm/src/vm/execute/scalar.rs`
- `crates/fpas-vm/src/vm/hosted/args.rs`
- `crates/fpas-vm/src/vm/hosted/mod.rs`
- `crates/fpas-vm/src/vm/hosted/proc.rs`
- `crates/fpas-vm/src/vm/layouts.rs`
- `crates/fpas-vm/src/vm/tasks/groups/registry.rs`
- `crates/fpas-vm/src/vm/tests/runtime.rs`
- `docs/future/improve-syntax/ap02-structured-diagnostics/01-audit-and-code-scheme.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/02-shared-diagnostic-record.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/03-project-and-build-diagnostics.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/04-json-output.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/05-diagnostics-reference.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/06-parameter-declaration-diagnostics.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/README.md`
- `docs/future/improve-syntax/ap02-structured-diagnostics/implementation-audit.md`
- `docs/pascal/language/basics/comments.md`
- `docs/pascal/language/functions/parameters.md`
- `docs/pascal/program-structure/cli.md`
- `docs/pascal/program-structure/projects.md`
- `docs/pascal/std/host/proc.md`
- `docs/pascal/std/result/option.md`
- `docs/pascal/std/result/result.md`
- `docs/pascal/std/testing/test.md`
- `docs/pascal/tools/README.md`
- `docs/pascal/tools/debugger-dap.md`
- `docs/pascal/tools/debugger-jsonl.md`
- `docs/pascal/tools/debugger.md`
- `docs/pascal/tools/diagnostics.md`
- `docs/pascal/tools/editor-integration.md`
- `editors/vscode/BUG_REPORT.md`
- `editors/vscode/README.md`
- `editors/vscode/src/workflow/diagnostics.ts`
- `editors/vscode/test/debugger_host/runtime_failure_filters.ts`
- `editors/vscode/test/extension.test.ts`
- `editors/vscode/test/fixtures/fixture-index.json`
- `editors/vscode/test/semantic_tools.ts`
- `editors/vscode/test/workflow/host.ts`
- `editors/vscode/test/workflow/unit.ts`
- `examples/README.md`
- `tests/manual/assert_fail_demo.fpas`

Removed during module splits:

- `crates/fpas-cli/src/cli_output.rs`
- `crates/fpas-cli/src/main_tests/diagnostics.rs`
