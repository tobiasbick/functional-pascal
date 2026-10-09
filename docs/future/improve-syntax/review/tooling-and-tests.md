# Tooling and test findings

See [scope and verification](README.md). The diagnostics, completion and host-test defects below are independent of the compiler failures.

<a id="t01"></a>

## T01 — P2: LSP project failures discard the stable diagnostic record

**Package:** AP02.

**Requirement:** [docs/future/improve-syntax/ap02-structured-diagnostics/implementation-audit.md:38-44](../../../../docs/future/improve-syntax/ap02-structured-diagnostics/implementation-audit.md) states that source identity is retained and errors are not reconstructed from Display output. `02-shared-diagnostic-record.md:13-17` says missing positions stay absent and editor adapters convert the records. The completion map explicitly includes language service and LSP.

**Implementation:**

- [crates/fpas-language-service/src/analysis/project.rs:47-53](../../../../crates/fpas-language-service/src/analysis/project.rs) and `108-110` convert `ProjectError` into `LanguageServiceError::Analysis` using `message.to_string()`.
- [crates/fpas-language-service/src/error.rs:33-40](../../../../crates/fpas-language-service/src/error.rs) stores only the resulting message plus a path for an analysis failure.
- [crates/fpas-lsp/src/diagnostics/publisher.rs:116-128](../../../../crates/fpas-lsp/src/diagnostics/publisher.rs) emits `FPAS_ANALYSIS` / `FPAS_PROJECT_IO`, `Range::default()`, and `error.to_string()` instead of preserving the original diagnostic.

**Reproduction:** Create `demo.fpasprj` next to `main.fpas` with this manifest:

```toml
[project]
name = "demo"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
```

Use this `main.fpas`:

```pascal
program Demo;
uses Demo.Missing;
begin
end.
```

Initialize the actual `fpas-lsp` binary with the project as `rootUri`, send `initialized`, then `textDocument/didOpen` for `main.fpas`. Verified with `python .temp-data/syntax-review-tooling/probe.py lsp`.

**Actual:** `textDocument/publishDiagnostics` contains `code: "FPAS_ANALYSIS"`, a zero-width range at `{line:0, character:0}`, and a message that embeds `error[FP4115]: Unknown unit ...` plus a rendered help line. CLI emits the structured `FP4115` identity. Consumers must parse prose to retrieve the code, and the editor marks the first character of the program heading.

**Expected:** Retain the project's original code, message, hint, and source attribution through the language-service layer; do not invent a source position for a genuinely unlocated failure. For this specific source error, also address T02 so that the import provides a real location. T01 exists independently of T02 because the adapter loses any project record passed to it.

**Impact:** Stable-code filtering, code actions, editor attribution, and consistent CLI/LSP diagnostics fail for project/graph failures even though ordinary lexer/parser/sema diagnostic conversion is structured.

**Tests needed:** Real LSP missing-unit, nonexported-unit, cycle, source-read, and dependency-manifest cases, with structured code and correct target document/range assertions instead of checking only message prose. Include a truly location-free failure whose location is not fabricated.

**Evidence:** `.temp-data/syntax-review-tooling/lsp-diagnostics.json`; minimal project in `missing-unit/`.

<a id="t02"></a>

## T02 — P2: Root-program import failures lose an available path and source span

**Package:** AP02; relevant to AP05 imports as a consumer.

**Requirement:** [docs/pascal/tools/diagnostics.md:32-40](../../../../docs/pascal/tools/diagnostics.md): “A `uses` entry that names a missing unit is located at the import.” AP02.3 states that orchestration retains authoritative source paths.

**Implementation:** [crates/fpas-project/src/unit_graph/resolve.rs:150-161](../../../../crates/fpas-project/src/unit_graph/resolve.rs) reduces root imports to string queue entries and creates `unknown_unit_error(..., "program")` without attaching source. The same function retains `node.path()` and `used.span` for unit dependencies at `168-179`. `validate_root_uses` at `25-35` likewise returns nonexported-unit errors without the root import span.

**Reproduction:** Use the same project as T01:

```text
fpas check .temp-data/syntax-review-tooling/missing-unit/demo.fpasprj --diagnostics json
```

**Actual (confirmed):** One `FP4115` record with `source: null` and `location: null`. The file and `uses Demo.Missing` token range are known in the parsed program.

**Expected:** `source` identifies `main.fpas`; the diagnostic span points to `Demo.Missing` on line 2 rather than discarding available provenance.

**Impact:** An agent/editor consuming CLI JSON cannot navigate to or fix a basic root-program import error. The issue occurs before LSP adaptation, so fixing T01 alone will not recover this information.

**Tests needed:** Real CLI `check`/`build`/`run` project-path and single-source root-import diagnostics, asserting source path and exclusive import range. Add a root nonexported-unit case as well; the similar loss is established by source inspection, while this audit dynamically reproduced the missing-unit case only.

**Evidence:** CLI record in `.temp-data/syntax-review-tooling/lsp-diagnostics.json`; the same minimal project as T01.

<a id="t03"></a>

## T03 — P3: Native completion fails for an incomplete chain over nested Result success types

**Package:** AP06 editor migration.

**Requirement:** [docs/future/improve-syntax/ap06-dot-call-targets/03-fixed-dot-resolution.md:40-41](../../../../docs/future/improve-syntax/ap06-dot-call-targets/03-fixed-dot-resolution.md) includes incomplete chains in editor coverage. The catalog is intended to supply completion by the receiver's static type, including type-changing chains.

**Implementation:** [crates/fpas-language-service/src/intellisense/native_receiver/inference.rs:32-37](../../../../crates/fpas-language-service/src/intellisense/native_receiver/inference.rs) splits a Result type at the first comma, without accounting for a nested Result in its success argument. For `Result of Result of string, integer, boolean`, this treats `Result of string` as the success type and `integer, boolean` as the error type; both become `Ty::Error`.

**Reproduction:** Open this incomplete program and request `textDocument/completion` immediately after the final dot:

```pascal
program Demo;
begin
  const Value: Result of Result of string, integer, boolean := Ok(Ok('x'));
  discard Value.Unwrap().Unwrap().
end.
```

Verified with `python .temp-data/syntax-review-tooling/editor_probe.py`. The request position is zero-based `{line:3, character:34}`.

**Actual:** Empty completion result.

**Expected:** The 31 string instance operations, including `Length`, `Slice`, and `IsEmpty`. Replacing the final dot with `.Length();` produces a valid program and exits successfully through the actual CLI. A control using `Result of Option of string, integer` and the same two `Unwrap()` calls correctly returns all 31 string operations. A longer array/Map/Find/Unwrap chain also returns the string operations correctly.

**Impact:** An ordinary valid nested container type loses type-specific completion exactly when the user is entering the next native operation. Runtime checking/lowering of the completed form succeeds.

**Tests needed:** Completion on incomplete native chains with Result nested on either side, multiple levels, aliases, and a callback whose result contains nested Result. Reuse recursive type syntax rather than splitting type text at the first delimiter.

**Evidence:** `.temp-data/syntax-review-tooling/editor-results.json`; `editor/case1.fpas`; valid completed control `editor/complete1.fpas` (run exit 0).

<a id="h01"></a>

## H01 — P2: Resolved — workflow diagnostics use the fixture token range

**Package:** AP16 consumer migration; AP02 editor verification.

[Workflow host coverage](../../../../editors/vscode/test/workflow/host.ts)
derives the expected source range from `MissingCall` in the generated invalid
program. It verifies FP3003, severity, source, both range endpoints, and the
help text. The source position assertion remains exact when the declaration
spelling changes.

The [workflow unit fixtures](../../../../editors/vscode/test/workflow/unit.ts)
use native paths for remembered-project selection. Explicit Windows and POSIX
path-identity controls remain in the
[project-index tests](../../../../editors/vscode/test/workflow/project_index.ts).

<a id="h02"></a>

## H02 — P2: Resolved — debugger-host sources use current block and pattern syntax

**Packages:** AP13 and AP20 consumer migration.

The positive generated programs in
[debugger-host coverage](../../../../editors/vscode/test/debugger_host/)
use named function, enum, case, conditional, and loop closers. Case patterns
use `when` and explicit `const` bindings. The migration preserves source line
counts and breakpoint locations in all affected scenarios.

Independent extraction and `fpas check --std-lib lib <fixture> --diagnostics json`
verify all 33 positive embedded programs in 31 source files, including the
[reference-call coverage](../../../../editors/vscode/test/debugger_host/var_parameters.ts).
The fixtures cover breakpoint policies, pause, closures, forced return, frame
restart, function values, live reload, payloads, tasks, initialization, and
variant manipulation. Intentionally invalid evaluation expressions and
rejected debugger operations retain their negative assertions.

The shared debugger call boundary rejects missing explicit `var` arguments
before the callee body executes. Rust and real DAP tests cover unused parameters,
callee-body failures, procedures, instance methods, function values, and bound
methods. Accepted explicit reference calls operate on detached storage;
stopped-state assignments write through reference parameters. These repairs are
complete in the
[compiler follow-ups](../../compiler-panic-followups.md#reference-parameters-in-debugger-calls-and-assignments).

See [repair verification](README.md#h01h02) for the complete host and Rust
check results.

## Additional successful tooling checks

All 156 positional/named native-catalog programs compiled and lowered. 146 executed successfully as written; ten semantic-only fixture inputs intentionally violated runtime bounds. Using valid bounds for those ten produced ten successful runs. This exercises all 78 catalog entries, but is smoke coverage rather than an assertion of every operation result. Real LSP transcripts confirmed T01 and T03 with successful control cases. Known debugger limitations for named arguments, typed record construction and `var` calls are already tracked in [compiler follow-ups](../../compiler-panic-followups.md); they are not counted again as new findings.
