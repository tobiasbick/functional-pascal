# AP35.1: Condition decisions

Package: [AP35](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Record separate user decisions for pattern predicate contexts and the
postcondition binder. For `is`, compare transparent grouping alone with a
broader Boolean form; specify bindings under conjunction, disjunction,
negation, guards, loops, and value use wherever allowed. Reject unsafe binding
paths explicitly. For `ensures`, select a grammar that cannot reinterpret a
parenthesized identifier as a different operation accidentally.

## Prerequisites and affected areas

AP20/AP23 are complete. Inspect their current parser/sema behavior and
`docs/specs/grammar.ebnf`. Change planning files only. The contract expression
allowlist remains the implemented AP23 boundary, not a purity proposal.

## Documentation and verification

Do not change the current handbook at this step. Provide accepted/rejected
examples for grouped and ungrouped tests, binding/nonbinding patterns,
procedure postconditions, and function result names. Both implementation
work packages require their respective decisions to be recorded first.
