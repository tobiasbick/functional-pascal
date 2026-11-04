# CLR CLI gaps

Only open CLI work is listed here. The [CLI reference](../pascal/program-structure/cli.md)
owns usable commands. Proposed names below do not select an implementation or
change FPAS syntax.

## Command inventory

| Proposed command | Open work |
| --- | --- |
| `init project`, `init library`, `init workspace` | Create program/library `.fpasproj` scaffolds and mixed `.slnx` solutions, with path/name selection, preview, structured results and preservation of existing files. |
| `build` | Provide an FPAS frontend for MSBuild with explicit source/project/solution selection and forwarding rules. Application publishing is separate work. |
| `run` | Select, build and launch a source, project, solution or assembly. Define arguments, streams, cancellation, working directory and child exit status. |
| `test` | Discover and execute FPAS test programs with filtering, listing, isolation, deadlines, cancellation, output and structured results. Select test conventions or a CLR test-framework integration. |
| `fmt` | Format the current language while preserving comments and meaning; support validation, changed-file listing and stdout output. |
| `doc` | Look up declarations, signatures and documentation, including referenced assemblies' XML documentation. |
| `refactor` | Prepare checked source edits and apply explicit plans. See the operations below. |
| `env --json` | Report compiler, FPAS SDK, .NET and available language-server/debugger installations. |

Future commands need concise help/examples, structured failures, bounded work,
stream ownership and a cancellation contract. Short aliases remain unselected.

## Project and solution inputs

Build/run selection rules remain open for standalone sources, `.fpasproj`, `.slnx`
and built assemblies. Standalone sources need reference resolution and temporary
output ownership beyond explicit `compile` inputs. Reuse evaluated
[workspace contexts](../architecture/workspaces.md) and MSBuild dependency data.

Solution execution needs an explicit startup rule and actionable ambiguity errors.
A mixed solution may launch a C# or F# host consuming an FPAS library. Project
selection must not assume that every FPAS project is executable. Define argument
forwarding after `--` without consuming the application's own options.

Application apphosts, self-contained/single-file publishing, AOT and signing need
separate SDK/CLI contracts. Compiler distribution belongs in
[distribution gaps](dist-gap.md).

## Refactorings

| Proposed refactoring | Open work |
| --- | --- |
| `rename` | Resolve a symbol, check conflicts, and prepare declaration/use edits. |
| `add-parameter` | Validate declarations and affected calls, parameter modes, argument binding and evaluation order. |
| `extract-variable` | Preserve scope, evaluation order and execution frequency. |
| `organize-imports` | Preserve aliases, comments and initialization effects while removing safe unused imports. |
| `apply` | Revalidate exact planned edits against current inputs, reject stale plans and report publication results. |

Refactorings need typed fragment/range validation, context/deadline selection,
edit ownership, plan output and explicit application. Share compiler/workspace
analysis rather than building separate resolvers.

## Diagnostics, tests and profiling

- `check --fix` needs [diagnostic edit data](diagnostics-gap.md), safe edit
  ownership, bounded rechecking and publication.
- Test programs need assertions, skip/strict semantics, scratch directories and
  golden-output rules. Multiple program entries cannot share one ordinary CLR
  executable project. Interactive terminal tests need a separate controlled
  terminal contract.
- A profiling frontend, if selected, needs .NET collection/summary behavior.
  Runtime-failure reports need exception/source stacks and debugger/dump data;
  arbitrary frame locals cannot be recovered from stderr alone.
- Tracepoints and backend-dependent trace/thread inspection need their own
  scope and evidence before exposure as FPAS commands.
- General command reports need schemas; existing command-specific JSON does
  not define the formats of future commands.
- Disabling contracts would require an explicit language/runtime policy decision.

[D1](documentation-gaps.md#d1-reproducible-benchmarks) and
[D3](documentation-gaps.md#d3-profiling-and-dump-walkthroughs) track verified
measurement and profiling/dump procedures. Accept those workflows on Windows and
Linux with the corresponding tooling work.

## Scope

LSP and editor workflows belong in [editor gaps](editor-gap.md). `fpas-bench`,
`--std-lib`, replacement `Std.*` wrappers and VM-specific execution/GC controls
are excluded. Batch debugging, command-file runners and alternate public debugger
transports are excluded. Clean/restore/publish wrappers and a profiling command
remain unselected.
