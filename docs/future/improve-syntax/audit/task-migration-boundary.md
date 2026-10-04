# Task syntax migration boundary

This audit records a sequencing dependency found while removing old type forms
from the generic-data work in [stage 4](../stages/04-functional-core.md). It does
not change the [language contract](../language-contract.md).

## Reproductions at the generic-data checkpoint

Three standalone CLI checks isolated the behavior before local binding inference:

| Source form | Check result | Current owner |
|-------------|--------------|---------------|
| `var Pending: task := go Work();`, where Work is a procedure | Accepted, exit 0 | Bare task result inference |
| `var Pending: task of (unit) := go Work();` | Rejected, exit 1; F1002 at `unit` | There is no source-level unit type |
| `var Pending := go Work();`, where Work returns integer | Rejected, exit 1; F1001 at `:=` | Local annotations are still required |

Parser `decl/type_expr.rs` accepts bare `task` as a named type; semantic type
resolution represents its result as an inference hole. Parser
`decl/data/const_var.rs` still requires a colon and explicit annotation. Existing
procedure-task consumers include the task-group lifecycle test and worker
pipeline example. They cannot all migrate to a concrete function-result type.

## Dependency

The contract forbids bare `task` and directs inferred local bindings to omit
their entire annotation. Uniform local inference belongs to the next stage-4
checkbox. Stage 5 changes procedure spawning to a statement without a result
handle and explicitly forbids a public unit-result type or bare-task exception.
It also owns the replacement of task-group and alternate spawning APIs.

Removing bare task syntax in the current generic-data slice therefore requires
either advancing those dependent facilities or explicitly assigning bare-task
removal to their coordinated migration. Adding `unit` as a source type, changing
procedures to functions, or silently retaining a permanent bare-task shortcut
would contradict the approved target.

## Approved boundary

Keep the target language unchanged. Complete the other generic-data type,
constructor and pattern migrations in the current slice. Assign removal of
bare `task` to the coordinated binding/task migration, after local inference and
procedure spawning without handles are implemented. Record it as a required
stage-5 migration dependency and verify that no bare task consumer or parser
branch remains at that boundary.

The user approved this boundary and continuation of the current generic-data
slice. Bare task removal remains mandatory at the coordinated binding/task
migration boundary. Other generic-data syntax conversion and removal may proceed.
The other generic-data changes and their verification are complete in the
[generic-data delivery](generic-data.md). Its owning checkbox is closed; removal
of bare task syntax remains required at the later coordinated boundary.

Local initializer inference is now delivered in stage 4. The remaining dependency
is stage 5 procedure spawning without handles and its structured task migration.
