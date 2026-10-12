# AP39.2: Coverage checks

Package: [AP39](README.md). Status: blocked by AP39.1.

## Scope and affected areas

Implement the agreed coverage/dead-arm contract. Inspect
`crates/fpas-sema/src/check/stmt/control_flow/if_case/coverage.rs`,
`exhaustiveness.rs`, pattern checking, expression case checking, diagnostics,
and branch lowering. Keep guard evaluation order and binding scope intact.

## Migration and documentation

Migrate selective cases to the chosen canonical form with preserved effects
and explicit no-op behavior. Update
[case introduction](../../../pascal/language/control-flow/case-of-intro.md),
[exhaustiveness](../../../pascal/language/pattern-matching/exhaustiveness.md),
scalar labels, guards, decision expressions, grammar, and compact reference.

## Verification

Exercise every cell of the recorded selector/position matrix, nested payloads,
guarded versus unguarded coverage, duplicate labels and overlapping ranges,
unreachable branches, and addition of an enum variant. Verify migration does
not evaluate a scrutinee twice or add unintended effects. Run applicable
compiler, runtime, documentation, and FPAS checks.
