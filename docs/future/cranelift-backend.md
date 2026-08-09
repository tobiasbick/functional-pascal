# Deferred Cranelift backend

## Status

Cranelift is deliberately deferred. The portable register VM remains the only production execution
engine, and no Cranelift dependency, feature flag, compiler path, artifact variant, or command-line
surface is to be added as part of the current performance work.

This note preserves the idea without turning it into an active implementation promise.

## Why it is parked

The quiet portable-register-VM acceptance already reaches a 1.593x geometric-mean throughput gain
over the pre-rewrite VM and a 2.958x gain for the headless TUI workload. The current TUI profile
still attributes broad cost to bytecode dispatch, value destruction, hosted numeric and callback
calls, and aggregate traversal. Those findings justify continued measurement, but not the immediate
complexity of a second execution engine.

Cranelift would introduce target-specific code generation, executable-memory management, a new
runtime calling convention, stack maps or equivalent root tracking, error-unwind integration, and a
second correctness surface for every bytecode operation. That cost should only be accepted for a
measured workload that cannot be solved cleanly in the interpreter.

## Re-entry conditions

Work may start only after an explicit user decision and all of these conditions are true:

1. A release profile on a representative FPAS application still identifies interpreter dispatch as
   a leading end-to-end bottleneck after cheaper VM and runtime fixes are exhausted.
2. The benchmark suite contains that representative workload and a saved interpreter baseline.
3. One initial mode is selected: JIT execution or ahead-of-time native compilation. The first slice
   must not attempt both.
4. The bytecode verifier, runtime value representation, call convention, hosted intrinsic boundary,
   task behavior, and panic diagnostics have documented mappings suitable for a second backend.
5. The first experiment has a deletion rule: if it does not deliver a meaningful end-to-end win or
   requires user-visible FPAS changes, remove it instead of retaining a compatibility layer.

## First implementation slice, if reopened

The first experiment should be Windows x86-64 only and remain behind an internal development entry
point. This is an experiment boundary, not a portability or support claim.

The smallest useful slice is:

1. Compile one already verified FPAS function at a time from the settled register instruction stream
   to Cranelift IR.
2. Support integer and Boolean constants, register moves, arithmetic, comparisons, conditional and
   unconditional branches, and direct calls.
3. Keep hosted operations, aggregates, strings, closures, tasks, exceptions, and unsupported
   instructions in the interpreter; use an explicit fallback boundary rather than partially
   duplicating their semantics.
4. Reuse the existing verifier as the admission gate. Generated code must never accept a program
   image that the interpreter rejects.
5. Compare the same workload through the interpreter and experimental backend for result values,
   output, diagnostics, and panics before measuring speed.

The experiment must live in a focused crate or module boundary. Compiler lowering must not import
Cranelift concepts, and the portable bytecode/artifact format must not contain target-machine code.

## Acceptance requirements

Reopening this work does not authorize a language change. FPAS syntax, semantics, standard-library
behavior, diagnostics, and source-visible numeric behavior must remain identical unless the user
separately approves an exact change.

A production proposal would require:

- positive, negative, and edge-case equivalence tests against the interpreter;
- release benchmarks saved and compared with `cargo bench-fpas` on unchanged workloads;
- explicit measurement of compile latency, steady-state throughput, and memory use;
- clean fallback or rejection for every unsupported instruction;
- no target-specific data in portable `.fpascp` program images;
- updated current documentation only after the backend actually becomes user-visible.

Until those gates are intentionally reopened, the correct action is to improve and profile the
portable register VM, not to scaffold Cranelift.
