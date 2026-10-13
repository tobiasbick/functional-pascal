# AP23.2: Postconditions

Package: [AP23: Preconditions and postconditions](README.md)

Status: complete.

## Scope

`ensures` clauses with an optional clause-local name for the complete return
value are checked on every ordinary or propagated return in all build modes.

## Prerequisites

- AP23.1 (clause machinery and reserved keywords).
- The agreed [propagated-return rule](README.md#propagated-returns-ap232).

## Implementation

- Parser: `ensures [(Name):] Expression;`. AP35.3 requires the colon to
  distinguish a result binder from a normal grouped Boolean condition.
- Sema: the name has the routine's return type and is visible only in its
  clause; rejected in the body and other clauses and in procedures. Same
  expression restrictions as AP23.1.
- Compiler: check after every `return`, including `try` error exits, and at the
  end of a procedure before the value leaves the routine. Bind the complete
  return value, including `Error(...)` and `None`; a missing return is still an
  error. Read current parameters without snapshots or rollback.
- A shared exit block is lowered in the heading scope before body-local
  declarations. Return expressions are evaluated once and their complete values
  pass through this block; missing returns remain visible to the IR verifier.
- Violation panic with routine name, clause text, parameter values, and the
  returned value.
- Public unit interfaces retain postconditions and their result names in
  definition order; interface hashes include clause changes.
- Formatter comments and clause order are preserved. Editor symbols, hover,
  definitions, and rename confine result bindings to their original clause.

## Affected areas

- Parser `ast/contracts.rs`, `ast/routines.rs`, and `parser/decl/contracts.rs`.
- Sema `check/contracts/postconditions.rs`, routine/method checking, and heading
  capture collection; immutable clause-only captures survive body shadowing.
- Compiler `lowering/contracts.rs`, ordinary/`try` return lowering, and implicit
  procedure completion. No new VM opcode or intrinsic is needed.
- Unit contract interfaces, formatter clause/comment handling, language-service
  result symbols, and parser/Sema/compiler/interface/formatter/CLI/editor tests.

## Migration

None.

## Documentation

- [Preconditions and postconditions](../../../pascal/language/functions/contracts.md),
  function index/declarations, keyword list, panic documentation, formatter
  style, and `docs/specs/grammar.ebnf`.

## Verification

- Every return path, early returns, checked `try` exits for `Result` and
  `Option`, current `var` values and preserved writes, named return value scope and type
  errors, missing return value, release-build checks, panic details.
- Direct, indirect, bound, static, nested, generic, and tail-position calls;
  clause order, aggregate return values, return-name/parameter shadowing, stored
  immutable captures, imported fresh/reused/rebuilt units, and native bundles.
- Argument and return expressions run once; completed writes remain observable
  after a grouped postcondition panic. Import aliases cannot be shadowed.
- `tests/runner/contracts/postconditions_test.fpas` exercises successful ordinary,
  bound-method, current-`var`, and `try` propagated `Result`/`Option` returns.

## Remaining work

None for AP23.2 or AP23. AP25.1 completed its assessment and the user closed
AP25 without implementation. The next optional decision is tracked in the
[central plan](../README.md#recommended-order).
