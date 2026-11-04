# CLR documentation example checker gaps

Build a reusable FPAS checker under `apps/doc-examples/` as a `.fpasproj`
application, with explicit context templates beside it and invocation through
the repository tests. Exact arguments, marker names and integration remain open.
The [documentation-example guide](../pascal/tools/documentation-examples.md)
owns the usable verification workflow.

## Application and integration

- Define the project, source/context layout, compiler/SDK invocation and
  bootstrap order from a clean environment. FPAS consumer projects need a
  resolvable SDK before they can build.
- Keep Markdown as the source of each example. Extracted sources are generated
  inputs, not independently maintained copies.
- Define which checks move from C# tests to the application without dropping
  existing execution or behavior assertions.
- Select a verified path through the current CLR compiler/SDK and generated
  programs. The proposed `fpas test` workflow remains a [CLI gap](cli-gap.md).

## Markdown instructions

Names below describe candidate instructions, not an agreed marker contract.

| Instruction | Open work |
| --- | --- |
| `pascal` | Define per-fence check-only versus execution selection. |
| `id=NAME` | Give examples stable per-document IDs for reports, includes and diagnostics. |
| `fragment context=NAME` | Expand explicitly named contexts and compile their fragments. |
| `run` | Define execution opt-in while retaining meaningful assertions/results. |
| `error=FPnnnn` | Validate expected compiler rejection without accepting another failure. |
| `project=NAME file=PATH` | Define file placement, fixture construction and project ownership beyond companion grouping. |
| `manifest=PATH` | Check documented `.fpasproj` manifests and supporting project/fixture fences. |

Reject unknown/repeated instructions, invalid/duplicate IDs and malformed or
unclosed supported fences. Define backtick/tilde handling and line-ending
normalization. A scan must not silently ignore malformed examples. Keep schematic
signatures, output and grammar in non-Pascal fences; do not relabel broken
executable examples to evade verification.

## Contexts and companion projects

- Require an explicitly named context with one insertion position. Do not try
  wrappers until a fragment happens to compile.
- Resolve named includes to current Markdown with clear missing/ambiguous
  reference failures and mapped source locations.
- Group programs, units, actual projects, references and explicit fixtures.
  Validate units independently and reject requests to execute them as programs.
- Validate relative paths, duplicate files and ownership. Isolate generated
  projects without overwriting repository sources.
- Contexts may supply stated prerequisites; they must not rewrite an example's
  expressions, calls, declarations or semantics.

## Compilation and execution

| Evidence | Required checker behavior |
| --- | --- |
| Valid compilation | Check each selected fence with its explicit context and resolved CLR references. |
| Expected compiler rejection | Require the intended diagnostic; reject success, other errors and failures originating only in a context/companion. |
| Successful execution | Run selected programs with deadlines, captured output and assertions or explicit result expectations. |
| Expected runtime failure | Distinguish successful compilation from the intended CLR exception/runtime outcome. |
| Failure location | Report Markdown path/line, example ID, stage and relevant diagnostic/runtime evidence. |
| Complete selection | Reject requests that execute no program, unintended extra programs, or silently skipped tests. |

Define execution limits, cancellation and child-process cleanup. A timeout is a
failure. Network, filesystem, process and interactive examples need controlled
fixtures or explicit compile-only classification. Executed examples retain
ordinary host access; the checker is not a sandbox.

## Reports and checker verification

Define default document roots including handbook, agent reference and README,
explicit document selection, repository/compiler/SDK identity, required references
and fresh owned output directories. Retain extracted sources and diagnostics.
Provide human-readable failures and JSON with per-example compile/run outcomes,
totals and nonzero failure status. Verify Markdown/source coordinate mapping,
paths and line endings on Windows and Linux.

Regression coverage must deliberately break markers, fences, IDs, contexts,
includes, paths, companions, expected diagnostics and runtime outcomes. Include
context-only failures, unexpected success, empty/skipped execution and
nonterminating examples. Prove that a successful report cannot hide ignored
inputs or the wrong failure.
