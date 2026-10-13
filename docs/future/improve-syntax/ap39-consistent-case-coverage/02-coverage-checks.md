# AP39.2: Coverage checks

Package: [AP39](README.md). Status: complete.
Prerequisite: [AP39.1's recorded contract](01-coverage-decision.md).

## Delivered behavior

Statements and expressions use one exhaustive selector policy. Enum, Option,
Result, Boolean, and subrange cases need explicit unguarded coverage and forbid
else. Integer/string cases require else, including after a machine-wide integer
range. Distinct selectors follow their supported base type while retaining typed
labels; aliases and imported types retain the same rules. Unsupported selector
types and existing label/type/constant restrictions remain unchanged.

Coverage is recursive through payload patterns. Payload binders and ignores cover
their complete position; nested finite domains preserve correlations between
fields. Subranges use exact interval partitions based on label boundaries,
without per-value enumeration or a semantic domain-size cutoff. Integer endpoint
arithmetic uses widened internal values, including at both machine limits.

Guards never establish coverage, even constant-true guards. Duplicate labels in
one arm are rejected by matched values, including equivalent constants,
singleton ranges, and ranges equivalent within a finite selector domain. Labels
fully covered by preceding unguarded labels or their union are unreachable.
Partial overlaps remain legal. Selector evaluation once, label/guard order,
per-label guard re-evaluation, scalar guard-binding limits, and case binding
scope are preserved. No additional pattern syntax or guard proof is introduced.

FP3011 reports missing finite coverage or a required integer/string fallback;
FP3035 rejects finite else; FP3033 reports duplicate or unreachable labels,
including empty ranges and values outside a subrange domain. Repairs recommend
explicit missing arms, no-op null bodies, the required scalar else, or existing
if/is forms for selective handling.

## Implementation owners

- `crates/fpas-sema/src/check/stmt/control_flow/if_case/arms.rs` shares selector,
  label, guard, binding, and reachability checks between positions.
  `exhaustiveness.rs` owns the shared selector policy; `case_expression.rs`
  retains expression arm typing. `labels.rs` folds scalar range endpoints.
- `if_case/coverage/` splits the pattern matrix into constructor domains, row
  specialization, interval partitions, and witness generation. It replaces
  the former `coverage.rs` without a competing scalar coverage implementation.
- `crates/fpas-diagnostics/src/codes.rs` retains existing numeric codes;
  `scalar_bindings.rs` gives wildcard repairs consistent with finite coverage.
- `crates/fpas-compiler/src/lowering/case/` terminates the defensive no-match
  path for every accepted finite case, preserving functions that return from
  every arm. Existing selector, label, guard, and payload lowering is reused.
- Existing compiler source fingerprints invalidate incompatible compiled-unit
  sidecars. Exported finite bounds and typed constants use the existing unit
  interface format; no format change is needed.

## Migration, documentation, and editor

Selective integer/string statements in Notes, Explorer, Julia, Newton,
Mandelbrot, the guards example, and Rust fixtures have explicit no-op fallbacks
where unmatched values previously did nothing. Subrange fixtures and examples
list remaining ranges explicitly. Existing closed-variant standard-library
cases already follow the contract. Project manifests, source formatting,
standalone programs, and benchmark examples are checked without launching
interactive applications.

The [case introduction](../../../pascal/language/control-flow/case-of-intro.md),
[exhaustiveness](../../../pascal/language/pattern-matching/exhaustiveness.md),
scalar labels, ranges, guards, subranges, distinct types, diagnostics, grammar
semantic notes, compact reference, and FPAS authoring skill describe the delivered
matrix. VS Code supplies an open scalar case snippet with else and a complete
Boolean snippet without else. AP03/AP18/AP19/AP20/AP21 planning records reference
the replacement rules while retaining their completed status.

## Regression coverage

- `crates/fpas-sema/src/tests/stmt/case_coverage.rs` owns the statement/expression
  matrix, aliases, typed distinct values, guarded semantic duplicates, partial
  overlaps, interval unions, nested finite payload correlations, exact missing
  witnesses, extended domains, and machine-wide subrange limits.
- `tests/stdlib/language/case_coverage_test.fpas` protects selector evaluation
  once, guard order across and within arms, skipped guards, first-match behavior,
  no-op fallbacks, typed finite expressions, nested patterns, and fully returning
  Boolean/subrange statements through the compiler and ordinary runner.
- `crates/fpas-build/tests/incremental/coverage.rs` checks cold and warm unit
  reuse with import aliases, finite bounds, and typed constants. Extending an
  exported bound invalidates consumer coverage and identifies the missing value.
- `crates/fpas-language-service/tests/case_coverage.rs` verifies diagnostics
  against disk and changing editor overlays, including replacement and removal
  of stale errors without changing the source on disk.
- Existing parser, formatter, compiler, CLI, language-service/LSP, closed-enum,
  nested-pattern, type-identity, and compiled-unit tests retain their contracts.

Independent implementation review findings are resolved and rechecked.
No AP39 implementation or language decision remains open.
