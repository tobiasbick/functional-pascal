# AP35.2: Pattern conditions

Package: [AP35](README.md). Status: blocked by the pattern decision in AP35.1.

## Scope and affected areas

Implement the agreed pattern-condition contexts and binding flow. Inspect
`crates/fpas-sema/src/check/stmt/control_flow/conditions.rs`, expression
parsing, pattern checking, branch lowering, formatter precedence, and LSP
scope analysis. Preserve single evaluation and short-circuit order.

## Migration and documentation

Update [is tests](../../../pascal/language/pattern-matching/is-test.md), the
grammar, pattern/control-flow examples, and compact reference. Migrate only
forms affected by the recorded rule. Do not replace check-then-Unwrap across
effectful expressions by textual substitution; AP43 owns canonical guidance.

## Verification

Compare grouped/ungrouped execution, short-circuit effects, nested patterns,
and name availability across successful, failed, negated, and alternative
paths where supported. Check loops and nested scopes. Rejected contexts must
produce the selected canonical alternative. Run full applicable checks.
