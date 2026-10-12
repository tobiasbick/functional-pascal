# AP39.1: Coverage decision

Package: [AP39](README.md). Status: Decision pending; decision work package.

## Scope and prerequisites

Compare always-exhaustive case with existing selective scalar statements.
AP03/AP20/AP21 are the baseline. Record the user's coverage, else, and
unreachable-arm rules for enum, Option, Result, Boolean, integer, string,
subrange, and distinct types, including nested patterns and guarded arms.
Resolve any dependency on AP35's proposed guard contexts explicitly.

## Deliverable and affected areas

Planning files only. Document finite-domain reasoning limits, range overlap,
the cost of requiring no-op branches, and the benefit of detecting new enum
variants. Decide duplicate scalar labels even if broader case simplification
is rejected; do not mistake a current accepted label for an implementation
bug against today's specification.

## Documentation and verification

The current [exhaustiveness reference](../../../pascal/language/pattern-matching/exhaustiveness.md)
stays unchanged. A complete example matrix, including one-value Boolean
fallback and fully enumerated subranges, must have unambiguous outcomes.
