# P4 calls, frames, closures, and callbacks

P4 is complete on the inactive register-development path. The production CLI and stack VM remain
unchanged until the later cutover phase.

## Implemented pipeline

The AST lowerer discovers named routines and anonymous closures deterministically, assigns dense
`FunctionId` values, and emits typed `CallDirect`, `CallValue`, `MakeClosure`, `MakeCell`, `CellRead`,
and `CellWrite` operations. Parameters are pinned in declaration order. Captures follow semantic
analysis order; mutable captures use shared cells and nested mutable captures reuse enclosing cells.

The bytecode builder emits every IR function into one verified executable. Per-function metadata
records arity, capture count, register high-water mark, return convention, and code range. Arguments
and captures are copied into a reserved contiguous call window before the packed instruction.

The interpreter uses a flat register vector with explicit frame bases. A call saves the caller
`FunctionId`, instruction pointer, frame base, and absolute return destination, then appends a fresh
callee window. Arguments occupy the first registers and captures immediately follow. Return
truncates the callee window, restores the continuation, and writes the result. Call depth and total
live register slots have deterministic limits.

First-class register functions retain `FunctionId`, ordered captures, task-bound state, and a name
used only for diagnostics. Legacy stack chunks temporarily retain name-only values; the register
call path rejects them rather than performing name canonicalization or hash lookup.

`RegisterCallbackSession` is the P4 hosted-callback boundary. It invokes a `FunctionId` repeatedly
against shared immutable executable data, creates fresh frame state per invocation, survives a
callback panic, and rejects calls after cancellation or shutdown. This exercises the
`array_callbacks` shape without pulling general array opcodes forward from P5.

## Semantics and phase boundaries

No lexer, parser, grammar, or FPAS language documentation changed. Differential tests run the stack
and register paths for direct functions, procedures, recursion, early returns, nested routines,
named first-class functions/procedures, immutable anonymous captures, mutable capture cells, and
escaping named nested routines.

Method-shaped calls use the same ABI: the receiver is argument zero and the method is a numeric
function target. Record construction and field access remain P5, so P4 proves this ABI with a
preconstructed record instead of implementing aggregate opcodes early.

Globals, general aggregates, intrinsics, tasks, persistence, and CLI selection remain assigned to
later phases. `compile_register_subset` rejects them with a structured development-path diagnostic.

## Focused evidence

- `fpas-compiler::tests::register_subset::functions`: direct calls, procedures, recursion, early
  return, nested routines, and named first-class functions/procedures.
- `fpas-compiler::tests::register_subset::closures`: typed closure/cell bytecode plus immutable,
  mutable, anonymous, and escaping named nested closure differential tests.
- `fpas-vm::vm::register::tests::calls`: windows, return destinations, recursion limit, numeric
  values, capture order, cells, task-bound state, method ABI, arity, and invalid IDs.
- `fpas-vm::vm::register::tests::callbacks`: repeated callbacks, panic unwind, cancellation, and
  shutdown.
- `fpas-bytecode::register_bytecode::verifier`: invalid targets, arity, destinations, windows,
  capture counts, cross-function branches, and missing returns.

P4 records no production performance result. Benchmark history remains unchanged because the
production CLI still executes the stack VM and therefore does not exercise this implementation.
