# AP25.2: Pure function checker

Package: [AP25: Conservative purity](README.md)

## Scope

Add `pure function` declarations and the conservative checker for user code.

## Prerequisites

- AP25.1 (decision to implement).
- AP14.2, AP16.3, AP17.1.

## Implementation

- Lexer and parser: `pure` before `function`; reject `pure procedure` and
  `var` parameters on pure functions.
- Sema: pure functions call only pure functions; captures are immutable; local
  `var` bindings are allowed when their mutation cannot be observed outside
  the call; no I/O or globally mutable state. Purity is a capability of the
  callable type, exported through unit interfaces as verified metadata.
- Specify aliasing rules for shared handles and imported APIs.
- Diagnostics explain the conservative restriction that failed.
- AP23 contracts may call pure functions once this lands.

## Affected areas

- `crates/fpas-lexer/`, parser routine headings, a new
  `crates/fpas-sema/src/check/purity/` module, callable types,
  `fpas-unit` interfaces, closure capture checking.

## Migration

Rename identifiers spelled `pure`.

## Documentation

- New purity page under `docs/pascal/language/functions/`, contracts page,
  keyword list, `docs/specs/grammar.ebnf`.

## Verification

- Positive and negative calls, captures, aliasing, shared state, rejected pure
  procedures and `var` parameters, allowed local mutation and panics, imported
  pure functions.
