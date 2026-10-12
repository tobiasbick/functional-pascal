# AP23.1: Preconditions

Package: [AP23: Preconditions and postconditions](README.md)

Status: complete.

## Scope

`requires` clauses on named routine declarations are checked on every entry in
all build modes. Both `requires` and `ensures` are reserved; only preconditions
are implemented in this work package.

## Prerequisites

- AP07.3 (logical precedence), AP13.3 (routine closers), AP16.1 (constant
  classification).
- The agreed [expression and binding boundary](README.md#expression-and-binding-boundary).

## Implementation

- Lexer: reserve `requires` and `ensures`; diagnose their use as identifiers.
- Parser: `requires Expression;` clauses after the routine heading, for named
  routines and methods.
- Sema: Boolean typing; repeated clauses combine with short-circuit `and` in
  source order. Apply the agreed expression and binding boundary recursively,
  including omitted construction defaults; resolve native calls by identity.
- Compiler and VM: check on entry, including release builds and indirect
  calls through function values; a violation panics with routine name, clause
  text, and parameter values.
- Unit interfaces carry the clauses so imported routines are checked in the
  callee as compiled.
- Stored callable parameters/constants remain readable as values; forming new
  routine values or bound methods in a clause is rejected. Clause-only captures
  are collected before body-local shadowing.

## Affected areas

- Lexer keyword catalog; parser `ast/contracts.rs` and `parser/decl/contracts.rs`.
  Token-based parser callers supply the original source snapshot for clause text.
- Sema `check/contracts/`, routine/method checking, capture collection, and
  `interface/export/contracts.rs`.
- Compiler `lowering/contracts.rs` emits callee prologues and uses the existing
  VM panic/value-display path; no new VM opcode or intrinsic is needed.
- Unit `interface/contracts.rs` retains public clauses independently of callable
  type identity; canonical interface hashes include them.
- Formatter clause/comment handling, editor grammar and grammar fixtures;
  parser, Sema, compiler, interface, formatter, CLI, and ordinary FPAS runner tests.

## Migration

No repository FPAS identifiers required renaming. Reserved-name diagnostics
suggest replacements such as `RequiresValue` and `EnsuresValue`.

## Documentation

- [Preconditions](../../../pascal/language/functions/contracts.md), function
  index/declarations, panic documentation, keyword list, formatter style,
  diagnostic FP3037, and `docs/specs/grammar.ebnf`.

## Verification

- Entry checks, repeated clauses, boolean typing, rejected calls and `old`,
  release-build checks, panic details, indirect and imported calls, keywords
  as identifiers; violations do not become `Result` errors.
- Native receiver identity, mutable global/capture rejection, current `var`
  inputs, declaration-scoped omitted defaults, nested expression guards,
  stored callable reads versus new captures, and body-local visibility/shadowing.
- Direct, indirect, nested, generic, instance, static, and bound-method entries;
  repeated invocation, source-order short circuit, argument evaluation once,
  entry before body effects, aggregate/Unicode panic values, and missing returns.
- Independently compiled imported routines and bound methods, fresh/reused/
  rebuilt sidecars, public interface round trips and clause hashes, native bundles,
  formatter comment/idempotence checks, and `tests/runner/contracts/preconditions_test.fpas`.

## Remaining work

None for AP23.1. [AP23.2](02-postconditions.md) is also complete; it implements
postcondition/result-name syntax and ordinary/propagated-return checks.
