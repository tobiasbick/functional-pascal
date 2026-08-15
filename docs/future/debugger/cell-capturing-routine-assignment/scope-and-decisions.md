# Scope and decisions

Stable decision IDs remain valid across resumed sessions. Changing one requires
an entry in [progress.md](progress.md) with the reason and affected work IDs.

## Decisions

| ID | Decision |
|---|---|
| CCRA-D01 | Reuse the existing named-routine catalog resolution. No closure-expression parser, computed source expression, or display-text inference is added. |
| CCRA-D02 | A source is eligible only when verified metadata identifies its lexical owner and every capture source in runtime ABI order. Missing, duplicate, stale, or inconsistent metadata is an error. |
| CCRA-D03 | `Value`, `Cell`, and `EnclosingCell` are the only accepted direct capture kinds. `Cell` and `EnclosingCell` clone the exact existing `Arc` handle; they never copy, detach, replace, or inspect the cell payload while preparing assignment. |
| CCRA-D04 | The request frame must be the exact lexical-owner frame. The debugger does not search older recursive activations, another frame, another stopped task, or a similarly named routine. |
| CCRA-D05 | The destination is restricted to a source-declared mutable function-typed frame register resolved in the same frame and task. Globals, closure-cell roots, aggregate descendants, inactive payloads, Dynamic storage, and evaluation-only values remain rejected. |
| CCRA-D06 | Every constructed function containing a mutable cell is `task_bound` and carries its owning runtime task identity. Invocation outside that task fails before entering the callee; `go` continues to reject it. This enforces the existing language rule at runtime and does not change FPAS semantics. |
| CCRA-D07 | Existing VM-created task-bound closures receive the same owner identity invariant. A debugger-only marker or a second callable representation is forbidden. |
| CCRA-D08 | Immutable `Value` captures retain the current bounded portable-type and capture-graph validation. A value capture containing a task, opaque handle, cell, or nested task-bound function remains rejected in this slice. |
| CCRA-D09 | Preparation is non-executing and non-mutating. It may clone immutable values and shared cell handles, but it does not call code, lock a cell, consult the scheduler, or resume a task. |
| CCRA-D10 | Source proof, task proof, signature proof, graph limits, and target proof complete before one existing root commit. Every failure preserves the old value, cell graph, stop state, and inspection generation. |
| CCRA-D11 | JSONL, DAP, and VS Code remain adapters over one VM policy. No custom request, capability flag, or editor-only construction rule is added. |
| CCRA-D12 | Diagnostics use stable machine error kinds plus concrete messages and hints. Rendered names and `<cell>` / `<function ...>` summaries are never identities. |
| CCRA-D13 | No FPAS grammar, syntax, static semantics, or normative language documentation changes are part of this package. Existing concurrency semantics are only reinforced at runtime. |

## Success definition

The package is complete only when all of the following hold:

1. Direct and transitive mutable captures preserve exact cell identity.
2. The assigned routine observes later writes through the original cell and
   writes are visible through every pre-existing closure sharing that cell.
3. A task-owned function cannot be invoked or spawned from a foreign task even
   if debugger mutation bypasses the source program's static data flow.
4. Unsupported targets and capture graphs fail before any live-state change.
5. Standard JSONL, DAP, and VS Code assignment surfaces have equivalent success
   and failure behavior.
6. Current debugger docs, tests, artifact validation, and the central deferred
   row are reconciled only after the implementation gates pass.

## Stop rules

- Stop if runtime task ownership cannot be checked without putting a portable
  task ID into `.fpascu` or `.fpascp`; ownership is runtime-only state.
- Stop if exact cell identity would require matching a display name, reading a
  rendered value, or synthesizing a replacement cell.
- Stop if a destination can escape to another task without the runtime owner
  check in `CCRA-D06`.
- Stop rather than accepting global, capture-cell, descendant, Dynamic, stale,
  or foreign-task destinations as an incidental consequence of generic commit.
- Stop if any adapter needs semantics different from the shared VM operation.
- Stop and request explicit agreement before changing FPAS language behavior or
  normative language pages.
