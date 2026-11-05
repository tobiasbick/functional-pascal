# CLR CLI gaps

Only open CLI work is listed here. The [CLI reference](../pascal/program-structure/cli.md)
owns usable commands. Proposed names below do not select an implementation or
change FPAS syntax.

Future commands need concise help/examples, structured failures, bounded work,
stream ownership and a cancellation contract. Short aliases remain unselected.

## Publishing

Application apphosts, self-contained/single-file publishing, AOT and signing need
separate SDK/CLI contracts. Compiler distribution belongs in
[distribution gaps](dist-gap.md).

## Refactorings

| Proposed refactoring | Open work |
| --- | --- |
| `add-parameter` | Validate declarations and affected calls, parameter modes, argument binding and evaluation order. |
| `extract-variable` | Preserve scope, evaluation order and execution frequency. |
| `organize-imports` | Preserve aliases, comments and initialization effects while removing safe unused imports. |

The remaining operations need their own typed fragment/range and behavior checks.
Extend the [rename plan/application service](../pascal/tools/refactoring.md) with
compiler-backed validation; retain context/deadline selection, edit ownership,
plan review and explicit application.

## Diagnostics, tests and profiling

- Interactive terminal tests need a separate controlled terminal contract.
  Test-framework extensions outside the documented VSTest baseline and
  Microsoft.Testing.Platform self-hosted applications need separate scope.
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

LSP and editor workflows belong in [editor gaps](editor-gap.md). Batch debugging,
command-file runners and alternate public debugger transports are excluded.
Clean/restore/publish wrappers and a profiling command remain unselected.
