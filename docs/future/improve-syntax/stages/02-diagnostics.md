# Stage 2: Diagnostics

Prerequisite: the [language contract](../language-contract.md).
Completion is tracked in the [steering plan](../README.md).

## Target

Extend `fpas-diagnostics`; do not create a parallel error system. Keep existing
`Fxxxx` codes and their meanings, including current phase assignments. Add unused
codes for new errors and a project/build range after auditing the registry.
The former proposal to rename all codes to `FPxxxx` is removed.

Every diagnostic has a code, severity, phase, source location where available,
message, and optional expected/found/hint information. An unavailable location
is null, not a fabricated line zero. Spans include start and end positions.
Text and JSON render the same diagnostic record and code.

Add `--diagnostics json` to `check`, `build`, `run`, and `test`. Emit UTF-8 JSON
Lines on stderr, one complete diagnostic object per line; program stdout is
unchanged. In this mode, tool progress and test summaries must not contaminate the
diagnostic stream. Program-written stderr remains a separate concern: capture and
forward it as a distinguishable program-output JSON event so it cannot masquerade
as a compiler diagnostic. Document this envelope and preserve text-mode behavior.
Locations use one-based lines/columns, exclusive end positions, and a documented
Unicode column unit; use Unicode scalar positions consistently across renderers.

No heuristic should invent an application-level decision. A superseded syntax
form receives the canonical replacement; a semantic ambiguity explains what
information is missing. Diagnostics are not a compatibility parser mode.

## Work

- [ ] Audit existing codes, lexer/parser recovery, semantic errors, project/build
  reporting, runner transport, and runtime source mapping; record exact paths.
- [x] Extend the shared schema and renderers, including unavailable positions,
  expected/found details, source identity, and deterministic JSON serialization.
- [x] Complete shared coded diagnostics for all project/build/linker failures.
- [x] Preserve successful-source warnings as structured records through project
  and build APIs.
- [x] Preserve source-read, lexer and parser diagnostics through project loading,
  dependencies, standard-library loading and graph/snapshot APIs.
- [x] Preserve compiler/parser records and native linker errors in `BuildError`;
  defer text rendering until `Display` and retain known producer source paths.
- [ ] Implement CLI selection and child-runner forwarding without losing exit
  status, duplicating diagnostics, or misattributing imported-unit errors.
- [x] Add the [shared diagnostics reference](../../../pascal/tools/diagnostics.md)
  with the implemented Rust API, schema, location rules, code inventory, and a
  wrong/corrected example; document debugger JSONL null positions.
- [ ] Extend the reference with CLI stream rules and program-output envelopes
  when the command/runner integration is implemented.
- [x] Test the shared model's Unicode ranges, null positions and JSON escaping,
  parser expected/found details, project source-read failures, VM diagnostic
  mapping, LSP point conversion and debugger null-location events.
- [ ] Complete end-to-end structured-output tests through lexer, parser, sema,
  project/build, runtime, all four CLI commands, and the actual runner process.

New language diagnostics ship with their owning stage, using this foundation.
In particular, cover grouped/comma-separated formal parameters, missing named
closers, obsolete imports/member calls, unused values, and missing `var` markers.

## Acceptance

A consumer can identify the code, phase, cause, and available source range without
parsing prose. Text and JSON agree; Unicode locations, no-source errors, multiple
errors, mixed program output, and runner exit failures are covered. Existing
diagnostic identities do not change just to match a new numbering preference.

Owners: `fpas-diagnostics`, diagnostic producers in lexer/parser/sema/compiler,
`fpas-project`, `fpas-build`, `fpas-linker`, `fpas-cli`, and `fpas-vm`; editor
adapters consume the same model through `fpas-language-service` and `fpas-lsp`.

## Implemented shared-model slice

The shared `Diagnostic` now has an optional span and optional expected/found
details. The parser's token expectation producer populates those details without
parsing prose. Existing Fxxxx codes retain their meanings; F5001 identifies a
source-read failure in the new project/build range.

`crates/fpas-diagnostics/src/source_range.rs` resolves UTF-8 byte spans to one-based
Unicode scalar coordinates and exclusive ends. Synthetic runtime spans are marked
as points: their unknown end remains null. Missing VM source-map entries produce
no span, including recording and invalid-register diagnostics; known runtime
locations retain source IDs. LSP point conversion uses scalar-to-UTF-16 mapping.

`crates/fpas-diagnostics/src/render/json.rs` serializes one deterministic JSON
record; the existing text renderer retains known-location output. Project source
reads use `crates/fpas-project/src/source/read.rs` to construct shared errors.
The project transport below carries these records to callers. Non-source
project/build failures are not all converted to shared diagnostics yet.

The implemented API is documented in
[shared diagnostics](../../../pascal/tools/diagnostics.md). The debugger JSONL
reference also documents null positions for unmapped runtime failures. Current
language syntax/semantics and FPAS source consumers are unchanged.

Regression coverage includes `fpas-diagnostics/tests/structured_output.rs`, code
range invariants, actual parser expectations, project source-read failure, VM
execution errors with source mapping, direct diagnostic construction without a
source-map entry, LSP Unicode point conversion and debugger null-location
events. Existing diagnostic consumers and their tests explicitly handle optional
spans. Full command-stream acceptance is not claimed by these tests.

Status: shared-model slice implemented and verified. The stage remains open.

Verification:

- `cargo fmt`, `cargo build` and
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  passed.
- `cargo test --workspace` passed: 3,306 tests, none failed or ignored.
- `fpas test tests/suite.fpasprj` passed: 458 passed, one skipped, none failed.
- Plan/reference links and `git diff --check` passed.
- The additional `fpas fmt --check lib/ apps/ examples/ tests/` check reports
  eight existing, unchanged files. No FPAS source was edited in this slice:
  `examples/math/mandelbrot/mandelbrot_app.fpas`,
  `examples/network/tcp_parallel_echo_server.fpas`,
  `examples/pascal/tui/notes-headless/notes_headless_benchmark.fpas`,
  `lib/Std/Json/Fields.fpas`,
  `lib/Std/Tui/Runtime/Application/ConsoleInput.fpas`,
  `tests/stdlib/fs/fs_create_dir_all_test.fpas`,
  `tests/stdlib/json/json_fields_typed_access_test.fpas`, and
  `tests/stdlib/toml/toml_fields_typed_access_test.fpas`.

## Implemented build-error transport slice

`crates/fpas-build/src/engine/error.rs` owns `BuildError` and `BuildDiagnostic`.
Compiler diagnostics retain their original codes, spans and details; artifact
parsing retains every lexer/parser record on failure instead of discarding all
but the first. Imported-unit errors retain the unit path when their source ID
matches. Artifact compiler errors retain the supplied main path. AST-based root
build/check calls do not infer a path from a possibly unrelated unit graph.

Linking failures retain the native `LinkError`, exposed through `link_error()`
and the standard error chain. Existing text-only build failures remain explicit;
this slice does not invent shared codes for unconverted error producers.

Regression tests in `crates/fpas-build/tests/diagnostics.rs` exercise public
build/check and artifact calls. `src/engine/error/tests.rs` covers an actual link
failure, foreign/missing source attribution and text-only failures.

Verification: `cargo fmt --check`, `cargo build`, strict workspace Clippy and
`cargo test --workspace` passed (3,313 tests, none failed or ignored). This includes
seven new build-diagnostic regressions and the existing CLI source-path test.
Documentation links and `git diff --check` passed. No FPAS source was changed;
the existing FPAS suite coverage ran through the workspace's CLI tests.

## Implemented project-source transport slice

`crates/fpas-project/src/source/error.rs` owns `ProjectError`. Its diagnostic
records and source path survive the parse cache, project dependency traversal,
standard-library loading, graph construction and node snapshot parsing. Source
read errors keep F5001; invalid UTF-8 has F5002 with no fabricated position.
Lexer/parser failures retain every record in producer order. Graph construction
uses per-file source ID zero; snapshot parsing retains the assigned graph ID.

The build snapshot boundary converts these errors into `BuildError` records,
including known paths on positionless errors. CLI, editor and distribution
adapters explicitly render at their existing text interfaces. Manifest and graph
validation messages, successful-source warnings and other uncoded producers
remain outside this completed slice.

`crates/fpas-project/tests/diagnostics.rs` covers main/dependency sources,
standard-library loaders, graph reads, invalid UTF-8, multiple parser errors,
snapshot source IDs and text-only manifest failures. Build regressions also
exercise conversion without losing records or positionless source paths.

Verification: `cargo fmt --check`, `cargo build`, strict workspace Clippy and
`cargo test --workspace` passed (3,322 tests, none failed or ignored). Nine new
regressions cover project-source transport and conversion to build errors.
Documentation links and `git diff --check` passed. No FPAS source files changed;
the workspace's existing CLI tests exercised the FPAS suites.

## Implemented coded project/build/linker failures slice

`ProjectError` and `BuildError` no longer have text-only variants; every failure
carries at least one coded record with message and optional hint. Embedded
`help:` lines moved into the hint field. New codes F5003–F5023 cover manifests,
paths, globs, duplicates, dependencies, unit graphs, workspace discovery and the
standard library; F5024–F5025 cover artifact I/O and encoding; F5026–F5034 cover
linker categories through `LinkError::code()`; F9003 marks project/build
invariant failures. Existing F5001/F5002 are reused for build-time source reads
and invalid UTF-8, and F5019 for sources changed during a build.

Manifest and graph records stay positionless without a source path because
their messages name the manifest or several files. Unknown and non-exported
units in a unit's `uses` clause are located at the import with the unit's path.
`crates/fpas-project/src/manifest.rs` owns shared manifest read/parse/value
failures. Workspace discovery functions return `ProjectError`; CLI and editor
adapters render them at their existing text interfaces.
`stage_standard_library` returns `BuildError`; `DistributionError` was removed.
Text output gains the `error[Fxxxx]:` prefix and separate `help:` lines.
Unknown-unit hints list at most ten units nearest the missing name's namespace
(`crates/fpas-project/src/unit_graph/unknown_unit.rs`) instead of every
standard-library unit.

Regressions: `crates/fpas-project/tests/diagnostics.rs` covers manifest read,
TOML syntax, field value, path and glob codes, dependency cycles, workspace
discovery and a located unknown import with JSON range.
`crates/fpas-build/src/engine/error/tests.rs` covers linker records and coded
text rendering; incremental and program-artifact tests assert F5019.

Verification: `cargo fmt --check`, `cargo build`, strict workspace Clippy and
`cargo test --workspace` passed (3,324 tests, none failed or ignored).
`fpas test tests/suite.fpasprj` passed: 458 passed, one skipped, none failed.
`git diff --check` passed. No FPAS source files changed.

## Implemented structured warnings slice

`fpas_diagnostics::FileDiagnostic` pairs a `Diagnostic` with an optional file
path and renders the text form. It replaces `fpas_build::BuildDiagnostic` as the
`BuildError` record type and carries `LoadedProject::warnings`.
`Diagnostic::warning_without_source` creates positionless warnings. New warning
codes F5035 (duplicate source file) and F5036 (skipped `program` source) carry
the source path and a hint; lexer/parser warnings keep their code and span.
The build pipeline produces no additional warning records: compiler and semantic
analysis currently emit only errors. CLI `check`, `build`, `run` and `fmt` print
warnings as `path: warning[Fxxxx]: ...`.

Regressions: `fpas-diagnostics/tests/structured_output.rs` covers warning text
and JSON; `fpas-project/tests/diagnostics.rs` covers coded loading warnings with
paths; CLI loading and process tests assert the warning codes.

Verification: `cargo fmt --check`, strict workspace Clippy and
`cargo test --workspace` passed (3,331 tests, none failed or ignored).
`fpas test tests/suite.fpasprj` passed: 458 passed, one skipped, none failed.
`git diff --check` passed. No FPAS source files changed.

Next: connect all four CLI commands and actual runner processes, separating program
stderr events and suppressing progress/test-summary contamination. Do not
advertise `--diagnostics json` until that complete path is verified.
