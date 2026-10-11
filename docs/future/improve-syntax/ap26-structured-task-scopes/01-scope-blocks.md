# AP26.1: Scope blocks and child completion

Package: [AP26: Structured task scopes](README.md)

## Scope

`scope ... end scope;` owns its directly started tasks. Every `go` inside a
scope starts a child owned by the innermost scope of the same routine; the
scope waits for its children on normal exit and cancels and waits on every
other exit (Q19, Q20). Cancelled
children stop without a token (Q24); ownership never crosses a routine
boundary (Q25).

## Prerequisites

- AP13.4 (statement-list blocks).
- AP17.1 (`var` arguments rejected for `go`).

## Implementation

- Lexer and parser: reserve `scope`; the scope statement with its own lexical
  scope.
- Sema: reject `go` inside an anonymous routine that is lexically inside a
  scope and does not open its own scope (Q25).
- Compiler: enter and leave the scope; leave with cancellation before
  `return`, `break`, `continue`, and `try` propagation out of the scope.
- VM: per-task stack of open scopes bound to the call frame that opened them;
  `go` enrolls its child only in a scope of the current frame, so nested
  routines and callbacks stay detached. Both `go` forms retain completion for
  the scope. Normal exit waits; cancelling exit requests cancellation and
  waits. A panic or runtime error leaving the task cancels and waits for all
  its open scopes before the task ends.
- Cancellation (Q24): scope children check their owner at instruction
  boundaries and wait points and end as `Cancelled`.
- No child survives a completed exit; its retained result is released.
- A pending cooperative scope exit retains its suspension until all children
  finish, both in the pool and in the debugger. Debugger cancellation joins
  the cancelled task's open scopes before publishing its terminal outcome.

## Affected areas

- `crates/fpas-lexer/`, parser statements, sema statement checking and
  closures, `crates/fpas-compiler/src/lowering/` (statements, control flow,
  concurrency), IR and bytecode opcodes with verification,
  `crates/fpas-vm/src/vm/tasks/` (a focused scopes module), debugger task
  driver, formatter, language service, editor grammar.

## Migration

Rename identifiers spelled `scope`. Existing detached `go` uses outside scopes
are unchanged.

## Documentation

- New task-scope page under `docs/pascal/language/concurrency/`, `go.md`,
  keyword list, `docs/specs/grammar.ebnf`.

## Verification

- One child first; then normal completion without cancellation, `return`,
  `break`, `continue`, `try` propagation, and panic with cancellation and
  waiting; nested scopes; both `go` forms; detached `go` outside scopes and
  in routines called inside a scope; rejected `go` in anonymous routines; no
  owned child running after exit.

Regression ownership: parser concurrency tests cover syntax and recovery;
formatter and language-service tests cover scope blocks in tooling. Compiler
`tests/concurrency/task_scopes.rs` covers emitted exit paths and detachment.
VM `tests/task_scopes.rs` covers cancellation at instruction boundaries;
debugger `tests/task_scopes.rs` covers cooperative completion and nested
cancellation. `tests/concurrency/task_scope_lifecycle_test.fpas` covers the
compiled test-runner lifecycle.
