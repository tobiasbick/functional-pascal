# Tooling and test findings

See [scope and verification](README.md). The diagnostics, completion and host-test defects below are independent of the compiler failures.

<a id="t01"></a>

## T01 — P2: Resolved — LSP project failures retain their diagnostic records

**Package:** AP02.

The [language-service error transport](../../../../crates/fpas-language-service/src/error.rs)
retains the original `ProjectError` and exposes all `FileDiagnostic` records in
producer order. Project discovery, dependency and standard-library loading, and
graph resolution preserve codes, severity, paths, spans, hints and expected/found
details. Source-read failures carry FP4101; invalid UTF-8 carries FP4102 and an
encoding correction hint. Both retain their available path without a position.

The [LSP project adapter](../../../../crates/fpas-lsp/src/diagnostics/project.rs)
publishes located records at the actual source URI using that source's snapshot.
Its [conversion](../../../../crates/fpas-lsp/src/diagnostics/convert.rs) retains
producer-local source IDs and structured details in `Diagnostic.data`. Records
without a usable source range use standard `window/logMessage` with their original
code and available path, without inventing a document marker. Root-program import
attribution is implemented by [T02](#t02).

The [ordered publication lane](../../../../crates/fpas-lsp/src/diagnostics/publication.rs)
clears obsolete related markers when an origin changes or closes, retains records
from other active origins, and checks editor versions and snapshot revisions.
Closing and reopening a source with the same client version cannot revive records
from its previous lifetime.

Fourteen new regressions cover original record preservation, parser details,
standard-library failures, file attribution, missing and nonexported units,
cycles, dependency manifests, correction, closing, shared origins and reopened
buffers. They include three
[language-service tests](../../../../crates/fpas-language-service/src/analysis/project_diagnostics.rs),
one conversion test, and ten real LSP process cases in the
[project-error tests](../../../../crates/fpas-lsp/tests/diagnostics/project_errors.rs)
and [root-import tests](../../../../crates/fpas-lsp/tests/diagnostics/root_imports.rs).
Existing [project-source tests](../../../../crates/fpas-lsp/tests/diagnostics/project_sources.rs)
also verify positionless read failures.

The [diagnostics handbook](../../../pascal/tools/diagnostics.md#editor-project-diagnostics)
and [editor integration](../../../pascal/tools/editor-integration.md) describe the
implemented behavior. See [repair verification](README.md#t01) for check results.

<a id="t02"></a>

## T02 — P2: Resolved — root-program import failures retain path and source span

**Package:** AP02; relevant to AP05 imports as a consumer.

The [project resolver](../../../../crates/fpas-project/src/unit_graph/resolve.rs)
retains original imports in its reachability queue. Missing-unit FP4115 and
nonexported-unit FP4116 failures carry the imported unit name's original span,
excluding an optional alias suffix. The
[public resolver API](../../../../crates/fpas-project/src/unit_graph/mod.rs)
accepts an explicit optional root path; unavailable paths remain absent while
the span and producer source ID are retained. Transitive failures keep their
importing unit's path.

[CLI program builds](../../../../crates/fpas-cli/src/project_build.rs) supply
the actual main or test-entry path. The
[program-artifact API](../../../../crates/fpas-build/src/program_artifact/mod.rs)
retains its supplied main-source path, including portable source metadata.
[Editor analysis](../../../../crates/fpas-language-service/src/analysis/project.rs)
supplies the current program snapshot's path, so the LSP publishes the original
code at the main document's real range and clears it after correction.

Fifteen new regressions cover six
[project resolver cases](../../../../crates/fpas-project/tests/diagnostics/root_imports.rs),
six [CLI cases](../../../../crates/fpas-cli/src/main_tests/diagnostics/imports.rs),
one [artifact API case](../../../../crates/fpas-build/tests/diagnostics.rs), and
two new [LSP process cases](../../../../crates/fpas-lsp/tests/diagnostics/root_imports.rs).
The existing missing-root LSP regression also checks its exact source and range.
Coverage includes source/project/workspace inputs, `check`/`build`/`run`, private
standard-library imports, aliases, CRLF and Unicode before the import, text/JSON
parity, serial and parallel test entries, absent root paths, source IDs,
disk and parsed overlays, transitive controls, and editor correction.

The [diagnostics handbook](../../../pascal/tools/diagnostics.md#project-error-transport)
and [editor integration](../../../pascal/tools/editor-integration.md) describe
the implemented behavior. See [repair verification](README.md#t02) for results.

<a id="t03"></a>

## T03 — P3: Resolved — incomplete native chains preserve recursive Result types

**Package:** AP06 editor migration.

Incomplete native chains use the receiver's recursive static type, including
nested `Result` success and error types, mixed containers, and callback results.
`Value.Unwrap().Unwrap().` supplies all 31 string operations for a
`Result of Result of string, integer, boolean` receiver. Native suggestions
retain catalog documentation and require no import edit.

The [parser type-fragment entry point](../../../../crates/fpas-parser/src/lib.rs)
uses the compilation-unit type grammar and rejects malformed or trailing tokens.
The [editor type mapper](../../../../crates/fpas-language-service/src/intellisense/native_receiver/types.rs)
resolves aliases recursively from their AST in each declaring unit's import
environment. The [callback inference](../../../../crates/fpas-language-service/src/intellisense/native_receiver/inference.rs)
maps anonymous return-type syntax directly and follows callable type aliases.
Alias traversal shares the bounded receiver lookup depth.

Sixteen new regressions include five
[parser cases](../../../../crates/fpas-parser/src/tests/decl/type_fragments.rs),
nine [language-service cases](../../../../crates/fpas-language-service/tests/intellisense/native_chains.rs),
and two [real LSP process cases](../../../../crates/fpas-lsp/tests/intellisense/native_chains.rs).
They cover nested success/error types, several levels, mixed arrays/options/dictionaries,
local and imported aliases, private aliases and import aliases in the declaring
unit, conflicting consumer names, named/anonymous callbacks, callable aliases,
named arguments, error-type signature help, malformed fragments, cyclic aliases,
parser nesting limits, diagnostic ordering, current editor buffers and exact
UTF-16 replacement ranges.

The [editor integration handbook](../../../pascal/tools/editor-integration.md)
describes the implemented behavior. See [repair verification](README.md#t03)
for results.

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

All 156 positional/named native-catalog programs compiled and lowered. 146 executed successfully as written; ten semantic-only fixture inputs intentionally violated runtime bounds. Using valid bounds for those ten produced ten successful runs. This exercises all 78 catalog entries, but is smoke coverage rather than an assertion of every operation result. The T01-T03 repairs have dedicated real LSP regression coverage. Named debugger arguments, typed record construction and `var` calls are complete in the [compiler follow-ups](../../compiler-panic-followups.md#completed-debugger-repairs); they do not add to the finding count.
