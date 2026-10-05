# AP13.6: Expression closers

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Close anonymous routines with `end function` / `end procedure` and record
updates with `end with`. Expressions take no terminating `;` of their own.

## Prerequisites

- AP13.3 (named routine closers).

## Implementation

- Parser: anonymous routine expressions end with their named closer; an
  anonymous routine argument ends before the call's `)` without `;`; record
  updates are `P with X := 1; end with`.
- Reject an extra `;` between an anonymous routine and the closing parenthesis.
- Formatter: expression closers inside arguments and declarations.

## Affected areas

- `crates/fpas-parser/src/parser/expr/` (closures, record update).
- `crates/fpas-fmt/src/emit/expr/`.

## Migration

Rewrite every anonymous routine and record update in all repository consumers.

## Documentation

- `docs/specs/grammar.ebnf` (`closure_expr`, `record_update`),
  `docs/pascal/language/functions/closures.md`,
  `docs/pascal/language/types/record-update.md`, `fmt-style.md`.

## Verification

- Expression endings in declarations, returns, and call arguments; nested
  anonymous routines; record updates inside arguments; the rejected extra `;`.
