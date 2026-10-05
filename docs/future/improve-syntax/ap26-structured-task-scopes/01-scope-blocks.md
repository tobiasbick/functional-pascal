# AP26.1: Scope blocks and child completion

Package: [AP26: Structured task scopes](README.md)

## Scope

Add `scope ... end scope;`. Every `go` inside a scope starts a child owned by
the innermost scope; the scope waits for its children on normal exit and
cancels and waits on every other exit (Q19).

## Prerequisites

- AP13.4 (statement-list blocks).
- AP17.1 (`var` arguments rejected for `go`).

## Implementation

- Lexer and parser: reserve `scope`; the scope statement with its own lexical
  scope.
- Compiler: register children with the innermost scope for both `go` forms;
  emit scope exit handling on normal completion, `return`, `break`,
  `continue`, `try` propagation, and panic.
- VM: lexical scope owner state reusing task-group cancellation and joining
  where their contracts fit; no child survives a completed exit.
- A nested routine called inside a scope does not inherit it; `go` there
  stays detached unless the routine has its own scope.
- Switching a `go` expression to a statement must not change its lifetime.

## Affected areas

- `crates/fpas-lexer/`, parser statements, sema statement checking,
  `crates/fpas-compiler/src/lowering/concurrency.rs`, control-flow lowering,
  `crates/fpas-vm/src/vm/tasks/` (a focused scope module), IR/bytecode
  verification, debugger mappings.

## Migration

Rename identifiers spelled `scope`. Existing detached `go` uses outside scopes
are unchanged.

## Documentation

- New task-scope page under `docs/pascal/language/concurrency/`, `go.md`,
  keyword list, `docs/specs/grammar.ebnf`.

## Verification

- One child first; then normal completion without cancellation, `return`,
  `break`, error propagation, and panic with cancellation and waiting; nested
  scopes; both `go` forms; detached `go` outside scopes; no owned child
  running after exit.
