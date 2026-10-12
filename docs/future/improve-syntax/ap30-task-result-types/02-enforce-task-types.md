# AP30.2: Enforce task result types

Package: [AP30](README.md). Status: blocked by AP30.1.

## Scope and prerequisites

Implement the recorded AP30.1 contract. Reject unresolved result types at the
source boundary; do not use the internal error type as an accepted wildcard.
Keep type inference and non-consuming observation distinct from result access.

## Affected areas

Inspect `crates/fpas-sema/src/check/name_resolution/types/resolve.rs`,
`types/compatibility.rs`, `std_registry/builtins/channel_task.rs`, and
`crates/fpas-sema/src/interface/`. Trace the chosen representation through
compiler lowering, unit interfaces, task-group builtins, and language services.
List the concrete files before implementation.

## Migration and documentation

Migrate affected declarations and standard signatures together with their
consumers under the [development process](../development-process.md#migration-rules).
Update the [task reference](../../../pascal/language/concurrency/task-handles.md),
[Std.Tasks](../../../pascal/std/concurrency/task.md), compact reference,
diagnostics, and grammar only when the rule is implemented.

## Verification

- The integer-task-to-string example fails during checking, before execution.
- Correct typed function and procedure tasks still check and run.
- Parameters, returns, nested containers, generic substitutions, fresh/reused
  unit interfaces, and group spawns preserve the result type.
- Heterogeneous observation cannot be used to retrieve a wrongly typed result.
- Run the applicable compiler, runtime, FPAS, and documentation checks from
  the development process; do not mark this package done from a probe alone.
