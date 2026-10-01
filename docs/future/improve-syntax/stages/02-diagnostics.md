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
- [ ] Preserve structured errors through project/build/linker APIs; the current
  source-read implementation still renders at the string-returning boundary.
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
reads use `crates/fpas-project/src/source/read.rs` to construct shared errors,
then render at the current string-returning project API boundary. This is not yet
structured transport through all project/build errors.

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

Next: preserve structured errors through project/build/linker APIs; connect the
mode to all four CLI commands and actual runner processes; separate program
stderr events from diagnostics and suppress progress/test-summary contamination.
Do not advertise `--diagnostics json` until that complete path is verified.
