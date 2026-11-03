# CLR documentation example checker gaps

This document records the missing general-purpose Markdown example checker for
the CLR toolchain. It is a gap inventory, not an implementation plan. It does
not authorize implementation, schedule work, select libraries, or change FPAS
source syntax.

The agreed location is `apps/doc-examples/`, with the checker written in FPAS
as a `.fpasproj` application. Its explicit context templates belong with the
application. It does not belong under `tools/`. The existing test suite would
invoke the application and check its results.

The Rust reference is the
[documentation example contract](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/tools/documentation-examples.md)
and the [FPAS checker](https://github.com/tobiasbick/functional-pascal/tree/0a4bc134ee71b5f922ebd0b1157ab55244bef657/tools/doc-examples).
Marker names below describe that reference or a possible CLR counterpart.
The exact CLR application arguments and marker contract remain open.

## Current CLR verification

Documentation examples are already checked in the CLR branch:

- [MarkdownExamples](../../tests/Fpas.IntegrationTests/MarkdownExamples.cs)
  extracts complete program, async-program, and unit fences. It recognizes
  `project=NAME` for companion sources.
- [LanguageSpecificationTests](../../tests/Fpas.IntegrationTests/LanguageSpecificationTests.cs)
  compiles complete examples from the handbook, agent reference, and repository
  README. Units compile as libraries. Programs execute with output or behavior
  assertions, and an unrecognized program needs an execution assertion.
- The existing tests also exercise generated PDB/XML outputs, selected error
  paths, and controlled filesystem/environment/network fixtures.
- [AgentReferenceTests](../../tests/Fpas.IntegrationTests/AgentReferenceTests.cs)
  checks that every agent-reference Pascal fence is a complete, uniquely named
  source, with explicit companions. Other documentation checks cover links,
  fragment marking, and copied inputs.

The current [documentation example guide](../pascal/tools/documentation-examples.md)
describes that scope. Syntax fragments are marked as incomplete; the
complete-source runner does not compile or execute them by inventing context.
Program-specific execution expectations currently live in C# tests.

The gap is a reusable FPAS application with explicit per-example instructions,
fragment contexts, expected-error checks, project fixtures, and portable reports.
It is not an absence of all Markdown verification.

## Application and test integration

`apps/doc-examples/` does not exist in the current CLR toolchain. Missing pieces
include the application project, its source/context files, its invocation/report
contract, and automatic execution through the repository tests.

The Markdown source remains the example being checked. Extracted source files
are generated test inputs, not independently maintained copies of the examples.
The application can use actual .NET APIs and packages; it does not require a
replacement `Std.*` library.

Including the application in `Fpas.slnx` is a possible integration point. The
[project layout](../architecture/project-layout.md) notes that FPAS consumer
projects need the built SDK to be resolvable first. Toolchain/bootstrap ordering
and invocation from a clean test environment remain integration gaps.

The Rust checker invokes `fpas check` and `fpas test`. Those commands do not yet
exist in the CLR CLI. A CLR checker needs a verified invocation path through the
actual CLR compiler/SDK and generated programs. The broader command gaps remain
in [CLI gaps](cli-gap.md); the Rust VM cannot validate CLR FPAS examples.

## Markdown instruction gaps

| Rust instruction | Current CLR status or missing counterpart |
| --- | --- |
| `pascal` | Complete sources already compile, and complete programs execute in the current tests. A general per-fence check-only mode is missing from the proposed application. |
| `id=NAME` | Missing stable per-document example identity for reports, context includes, and diagnostics. Current tests identify complete sources by document, fence index, and program/unit name. |
| `fragment context=NAME` | Fragment marking exists, but named context expansion and compilation of fragments are missing. |
| `run` | Missing per-fence execution opt-in in a general checker. Existing execution coverage is supplied by the C# documentation tests. |
| `error=FPnnnn` | Missing general compile/runtime negative-example validation against an expected CLR diagnostic. Rust diagnostic numbers are not CLR expectations. |
| `project=NAME file=PATH` | Companion grouping partially exists. General file placement, fixture construction, and per-example project ownership need a checker contract. |
| `manifest=PATH` | Missing support for checking a project manifest documented alongside its sources. CLR projects use `.fpasproj`, not the Rust TOML format. Supporting XML or other project/fixture fences need an explicit contract. |

The Rust default checks a Pascal example without executing it, and `run` opts
into execution. The current CLR tests execute complete programs by default.
Adopting a marker-driven checker must account for that difference without
silently dropping existing assertions or execution coverage. Which checks remain
in C# and which become application-owned is unresolved.

Unknown or repeated instructions, invalid identifiers, duplicate IDs, and
malformed/unclosed fences need explicit failures. The Rust checker accepts
backtick and tilde fences and normalizes line endings. Corresponding CLR
coverage is missing. An all-example scan must not silently overlook examples
because of malformed supported markers.

The reference has no `skip` marker. Intentionally schematic text, signatures,
output, and grammar use their appropriate non-Pascal fences and carry no
compilation claim. Relabeling broken executable examples as text would defeat
the intended coverage.

## Explicit contexts and companion projects

The Rust context model makes the prerequisites of a fragment visible. Its
templates contain exactly one `// @example` insertion line and can supply
imports, declarations, fixtures, or an enclosing routine. `// @include
PATH#ID` inserts a specifically named documented example. Optional companion
units supply additional source files.

The CLR checker has no equivalent template/include handling or validation.
Open gaps include:

- Explicitly named contexts with exactly one insertion position, rather than
  automatically trying different wrappers until a fragment compiles.
- Includes that resolve to the current Markdown source, with clear missing,
  ambiguous, or invalid-reference errors.
- Source mapping between an extracted fragment, its context, generated files,
  and the original Markdown path and line.
- Per-example groups of programs, units, actual `.fpasproj` files, references,
  and any explicitly documented supporting fixtures.
- Independent validation of units and rejection of attempting to execute a
  unit as a program.
- Relative output paths, file ownership, duplicate-file checks, and isolated
  generated projects without overwriting repository sources.

Contexts may supply stated prerequisites. They do not authorize rewriting the
documented example's expressions, calls, declarations, or semantics.

## Compile, execution, and expected-result gaps

The intended checks distinguish several kinds of evidence:

| Evidence | Missing general checker behavior |
| --- | --- |
| Valid compilation | Check every selected supported Pascal fence through the current CLR compiler, using its explicit context and resolved references. |
| Expected compiler rejection | Require failure with the specified diagnostic, and reject unexpected success, other errors, or errors originating only in a context/companion. |
| Successful execution | Execute only selected programs with bounded lifetime, captured output, and meaningful assertions or explicit result expectations. |
| Expected runtime failure | Distinguish compilation success from the intended runtime failure. CLR exceptions and runtime outcomes need a representation rather than assuming Rust VM diagnostic codes. |
| Accurate failure location | Report the Markdown path, source line, example ID, stage, and relevant compiler/runtime evidence. |
| Complete execution selection | Reject an execution request that ran no program, multiple unintended programs, or an ignored/skipped test. |

The Rust `run` mode uses strict test execution with a five-second timeout and
requires exactly one executed test program. The CLR counterpart lacks an
application-level execution policy, deadline handling, and child-process cleanup.
The exact limits remain open. A timeout or failed inspection cannot count as a
successful example.

Compilation proves type/syntax correctness. Successful execution proves only
what the example's assertions or explicit expectations establish. Neither
automatically validates arbitrary prose or printed output. Existing CLR output
and behavior checks remain useful evidence.

Deterministic, noninteractive examples can run automatically. Network,
filesystem, process, or interactive examples need controlled fixtures or an
honest compile-only classification. Existing isolated-directory and local-proxy
tests provide examples of controlled execution. The checker supplies no sandbox
or special restriction on an executed program's ordinary host access.

## Reports and selection

The Rust checker selects the handbook and repository README by default, allows
explicit document selection, and writes extracted sources, compiler diagnostics,
execution results, and `report.json` into a new output directory. Its report
contains a schema, totals, failure count, and per-example outcomes.

The CLR application lacks corresponding contracts for:

- Default Markdown roots, including the handbook, agent reference, and README,
  plus explicit selection of other documents.
- Repository root, compiler/SDK identity, required references, and output
  directory selection without relying on one machine's paths.
- Human-readable failure locations and a structured JSON report with explicit
  compile/execution outcomes, totals, and nonzero exit status on failure.
- Fresh output directories, ownership/cleanup, and retained diagnostics useful
  for reproducing a failure.
- Paths and line endings on Windows and Linux, including Markdown/source
  coordinate mapping.

The CLR application does not need the Rust `--std-lib` option. Platform and
dependency selection follow the normal .NET/FPAS SDK model.

## Verification of the checker itself

The Rust implementation has
[behavior regression tests](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/tests/runner/documentation_examples_behavior_test.fpas)
that deliberately break Markdown, contexts, compilation, and execution. The CLR
application will need equivalent evidence that its reports cannot succeed by
ignoring a source or accepting the wrong failure.

Missing coverage includes unknown/duplicate markers and IDs, malformed fences,
missing contexts/includes, invalid paths, broken companions, incorrect expected
diagnostics, unexpected success, context-only errors, runtime failures, skipped
or empty execution selections, and nonterminating examples.

These are gaps in validating the proposed checker. This document does not add
tests, create the application, change existing Markdown markers, or move the
current test helpers. Narrow entries when implementation and verification exist,
and keep the ordinary documentation-example guide aligned with usable behavior.
