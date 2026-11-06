# CLR CLI gaps

Only open CLI work is listed here. The [CLI reference](../pascal/program-structure/cli.md)
owns usable commands. Proposed names below do not select an implementation or
change FPAS syntax.

Future commands need concise help/examples, structured failures, bounded work,
stream ownership and a cancellation contract. Short aliases remain unselected.

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
- Tracepoints and trace-oriented thread analysis need their own
  scope and evidence before exposure as FPAS commands.
- General command reports need schemas; existing command-specific JSON does
  not define the formats of future commands.
- Disabling contracts would require an explicit language/runtime policy decision.

Current measurement and diagnostic procedures belong in
[benchmarks](../pascal/tools/benchmarks.md) and
[diagnostic workflows](../pascal/tools/diagnostic-workflows.md).

## Scope

Additional language-server services and editor workflows belong in [editor gaps](editor-gap.md). The
[debugging reference](../pascal/tools/debugging.md) owns DAP and batch contracts.
Clean/restore wrappers and a profiling command remain unselected.
