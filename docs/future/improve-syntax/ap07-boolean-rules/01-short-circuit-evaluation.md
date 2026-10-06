# AP07.1: Short-circuit evaluation

Package: [AP07: Boolean rules](README.md)

## Scope

Specify and implement left-to-right short-circuit evaluation for boolean `and`
and `or`. `xor` stays eager.

## Prerequisites

None.

## Implementation

- Verify the current behavior in compiler lowering, constant folding, the VM,
  and debugger expression evaluation.
- Lower boolean `and`/`or` with branches; move boolean lowering into a focused
  module instead of growing `crates/fpas-compiler/src/lowering/expr.rs`
  (490 lines at planning time).
- Debugger watch evaluation follows the same rule.
- Integer operand rules are owned by AP07.4, which restricts logical
  operators to booleans and migrates integer operations to eager `Std.Bits` calls.

## Affected areas

- `crates/fpas-compiler/src/lowering/expr.rs` and a new boolean lowering module.
- `crates/fpas-compiler/src/optimize/` (constant folding).
- `crates/fpas-vm/src/vm/debug/` (expression evaluation).

## Migration

Check repository sources for right operands with side effects whose skipping
would change behavior; record each case in the pull request and adapt it.

## Documentation

- `docs/pascal/language/basics/operators.md`: evaluation order and
  short-circuiting.

## Verification

- Execution tests with side-effect traces: skipped right operand for
  `false and X` and `true or X`; evaluated for the other cases; eager `xor`;
  a failing right operand that is skipped; `try` inside a skipped operand.
- Debugger evaluation tests for the same cases.

## Implementation result

- Boolean `and`/`or` lower to branches in
  `crates/fpas-compiler/src/lowering/expr/boolean.rs`. A hidden Boolean local
  carries the selected result into the continuation, including when either
  operand contains nested short-circuiting or `try`.
- Constant folding operates on the guarded IR instructions without moving or
  executing them. Existing VM branch instructions provide the runtime behavior;
  eager scalar bytecode operations remain available for compiler-generated
  comparisons and integer operations.
- The debugger's expression walker skips the unnecessary right subtree before
  resolving names, invoking calls, applying `try`, or spending its runtime
  evaluation budget. Parsing and whole-expression validation still apply.
- Execution coverage lives in `tests/runner/boolean/` and
  `crates/fpas-compiler/src/tests/control_flow/boolean.rs`; debugger coverage
  lives in `crates/fpas-vm/src/vm/debug/tests/evaluation/boolean.rs` and
  `crates/fpas-debug/tests/boolean_evaluation.rs`.
- The operator documentation and debugger handbook describe the implemented
  evaluation order. AP07.3 defines precedence; AP07.4 restricts logical
  operators to Boolean values and migrates the eager integer checks to `Std.Bits`.

### Migration review

An AST-based review of `lib/`, `apps/`, `examples/`, and `tests/` found no
existing right operand that needs extraction to preserve a write, callback,
task operation, or I/O effect. No consumer migration is required. The new
regression tests deliberately place writes and failures in right operands.

The existing observable queries that can now be skipped are:

| Sources | Right operand | Result of skipping |
|---------|---------------|--------------------|
| VM, concurrency, and TUI benchmarks; `examples/network/https_server.fpas` | `ParamCount()` | The first comparison already determines argument-count validity. |
| `tests/stdlib/tui/background_pending_test.fpas`, `background_responsiveness_test.fpas` | `MonotonicMillis()` | No deadline read after the preceding loop condition is false. |
| `examples/math/mandelbrot/mandelbrot_render.fpas` | `MonotonicMillis()`, `IsCancellationRequested(WorkerToken)` | No clock read outside the yield interval; no second cancellation read after cancellation is established. |
| `examples/math/julia/julia_compute.fpas`, `julia_render.fpas` | `IsCancellationRequested(Token)` | No cancellation read outside the polling interval or after the row limit is reached. |
| `examples/network/tcp_parallel_echo_server.fpas` | `ElapsedMillis(Started)` | No elapsed-time read after the lifetime is no longer ready. |
| `lib/Std/Http/Stream.fpas`, `Sse.fpas`; `lib/Std/Tui/Runtime/Application/Loop.fpas` | Hosted state existence or input-count queries | No registry/query read after the handle or queue guard already determines the result. |

The remaining calls are value queries: lengths, membership, text predicates,
geometry, menu selection, note selection, layout participation, and surface
comparisons. Their return values are unnecessary on the skipped branch, and
they perform no required external effects.
