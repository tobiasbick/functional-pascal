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

## H01 — P2: The migrated workflow fixture retains the old diagnostic offset

**Package:** AP16 consumer migration; AP02 editor verification.

[editors/vscode/test/workflow/host.ts:54-56](../../../../editors/vscode/test/workflow/host.ts) writes:

```pascal
program Workflow; begin const Value:integer:=MissingCall(); end.
```

The same file, line 99, asserts zero-based character **43**. `MissingCall` actually starts at **45** after `let` was replaced with `const`. The real CLI/LSP/workflow diagnostic correctly reports line 0, character 45 and code FP3003. The fixture was migrated in `c66326e23`; the literal expected offset was not.

**Reproduction:** `node editors/vscode/scripts/run-tests.mjs`. Both the initial run and an isolated retry fail at `verifyWorkflowHost` with actual character 45 versus expected 43; exit code 1. The isolated retry had no concurrent builds.

**Impact:** The full extension-host suite is currently red. [editors/vscode/test/extension.test.ts:166-167](../../../../editors/vscode/test/extension.test.ts) awaits the failing workflow check before starting debugger-host checks, so those later checks and the subsequent standard-library navigation/lifecycle checks do not run. Passing TypeScript compilation and grammar verification does not establish that the host suite passes.

**Required correction and validation:** Derive the expected range from the fixture token or update the exact verified offset. Then run the complete suite; do not remove or relax the source-position assertion. H02 identifies additional invalid sources that are hidden by this first failure.

<a id="h02"></a>

## H02 — P2: Debugger-host fixtures still use removed block and pattern syntax

**Packages:** AP13 and AP20 consumer migration.

A bounded extraction checked 31 complete embedded FPAS programs from 30 TypeScript files under [editors/vscode/test/debugger_host/](../../../../editors/vscode/test/debugger_host/). **20 programs across 19 files fail parsing with FP2001; 11 pass.** The only template substitution needed was the numeric live-reload value, replaced by `1`. These are positive debugger fixtures, not expected syntax-error tests.

| Source owner | Template starts at line | First observed parser error |
| --- | --- | --- |
| [editors/vscode/test/debugger_host/breakpoint_policies.ts](../../../../editors/vscode/test/debugger_host/breakpoint_policies.ts) | 21 | Expected `end while;`, found `end.` |
| [editors/vscode/test/debugger_host/capturing_routine_assignment.ts](../../../../editors/vscode/test/debugger_host/capturing_routine_assignment.ts) | 28 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/cell_capturing_routine_assignment.ts](../../../../editors/vscode/test/debugger_host/cell_capturing_routine_assignment.ts) | 28 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/forced_return.ts](../../../../editors/vscode/test/debugger_host/forced_return.ts) | 32 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/forced_return.ts](../../../../editors/vscode/test/debugger_host/forced_return.ts) | 159 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/frame_restart.ts](../../../../editors/vscode/test/debugger_host/frame_restart.ts) | 23 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/function_breakpoints.ts](../../../../editors/vscode/test/debugger_host/function_breakpoints.ts) | 21 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/function_value_assignment.ts](../../../../editors/vscode/test/debugger_host/function_value_assignment.ts) | 28 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/live_reload.ts](../../../../editors/vscode/test/debugger_host/live_reload.ts) | 102 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/pause.ts](../../../../editors/vscode/test/debugger_host/pause.ts) | 21 | Expected `end while;`, found `end.` |
| [editors/vscode/test/debugger_host/payload_mutation.ts](../../../../editors/vscode/test/debugger_host/payload_mutation.ts) | 28 | Expected `end enum;`, found `end;` |
| [editors/vscode/test/debugger_host/task_control.ts](../../../../editors/vscode/test/debugger_host/task_control.ts) | 30 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/task_debugging.ts](../../../../editors/vscode/test/debugger_host/task_debugging.ts) | 43 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/task_handle_assignment.ts](../../../../editors/vscode/test/debugger_host/task_handle_assignment.ts) | 28 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/task_lifecycle.ts](../../../../editors/vscode/test/debugger_host/task_lifecycle.ts) | 26 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/task_result_replacement.ts](../../../../editors/vscode/test/debugger_host/task_result_replacement.ts) | 29 | Expected `end function;`, found `end;` |
| [editors/vscode/test/debugger_host/uninitialized_assignment.ts](../../../../editors/vscode/test/debugger_host/uninitialized_assignment.ts) | 28 | Expected `end if;`, found `end.` |
| [editors/vscode/test/debugger_host/variant_construction.ts](../../../../editors/vscode/test/debugger_host/variant_construction.ts) | 35 | Expected `end enum;`, found `end;` |
| [editors/vscode/test/debugger_host/variant_replacement.ts](../../../../editors/vscode/test/debugger_host/variant_replacement.ts) | 28 | Expected `end enum;`, found `end;` |
| [editors/vscode/test/debugger_host/variant_transition.ts](../../../../editors/vscode/test/debugger_host/variant_transition.ts) | 28 | Expected `end enum;`, found `end;` |

The table shows the first diagnostic for each program; parser recovery may report additional errors. It is not a complete inventory of invalid constructs within each fixture. Examples include missing `end while;`, anonymous `end;` where `end function;` or `end enum;` is required, and old case arms such as `Choice.Count(Value):` / `Ok(Value):` without `when` and explicit `const` bindings.

**Reproduction:** Extract the indicated `program ... end.` template without altering its syntax and run `fpas check --std-lib lib <extracted-file> --diagnostics json`. Local evidence is in `.temp-data/syntax-review-tooling/embedded-editor/results.json`; the extraction harness is `.temp-data/syntax-review-tooling/embedded_editor_probe.py`. For example, `breakpoint_policies.ts:21` produces `Expected end while;, found end.` at the last program line.

**Impact:** Once H01 is corrected, these programs cannot reach the debugger behaviors their tests are intended to verify. They cover breakpoints, pause, closures, forced return, frame restart, function values, live reload, payloads, tasks, storage initialization and variant manipulation. Existing Rust debugger tests passing does not validate these separate TypeScript-generated programs.

**Required correction and validation:** Migrate all positive embedded sources, retain their intended runtime behaviors and breakpoint positions, and parse/check generated debugger programs independently before running the full host sequence. Negative parser fixtures must remain explicitly distinguished.

## Additional successful tooling checks

All 156 positional/named native-catalog programs compiled and lowered. 146 executed successfully as written; ten semantic-only fixture inputs intentionally violated runtime bounds. Using valid bounds for those ten produced ten successful runs. This exercises all 78 catalog entries, but is smoke coverage rather than an assertion of every operation result. Real LSP transcripts confirmed T01 and T03 with successful control cases. Known debugger limitations for named arguments, typed record construction and `var` calls are already tracked in [compiler follow-ups](../../compiler-panic-followups.md); they are not counted again as new findings.
