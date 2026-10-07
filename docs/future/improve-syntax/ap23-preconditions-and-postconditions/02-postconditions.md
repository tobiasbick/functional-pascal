# AP23.2: Postconditions

Package: [AP23: Preconditions and postconditions](README.md)

## Scope

Add `ensures` clauses with an optional clause-local name for the return value,
checked on every return in all build modes.

## Prerequisites

- AP23.1 (clause machinery and reserved keywords).
- The `try` error-return decision in the [package README](README.md#open-decisions).

## Implementation

- Parser: `ensures [(Name)] Expression;`.
- Sema: the name has the routine's return type and is visible only in its
  clause; rejected in the body and other clauses and in procedures. Same
  expression restrictions as AP23.1.
- Compiler: check after every `return` and at the end of a procedure, before
  the value leaves the routine; a missing return is still an error.
- Violation panic with routine name, clause text, parameter values, and the
  returned value.

## Affected areas

- Parser routine clauses, sema routine checking, compiler return lowering.

## Migration

None.

## Documentation

- The contracts page, `docs/specs/grammar.ebnf`.

## Verification

- Every return path, early returns, `try` exits (specify whether a `try`
  error return is checked and record it), named return value scope and type
  errors, missing return value, release-build checks, panic details.
