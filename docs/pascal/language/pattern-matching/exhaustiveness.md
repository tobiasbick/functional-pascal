# Exhaustiveness and reachability

- Language variants require exhaustive coverage in both
  statement and expression forms. Coverage descends into nested variant patterns.
- `bool` requires true and false, or a fallback.
- Integer, string, and CLR enum selectors require an unguarded catch-all pattern
  or `else`. Enum metadata does not rule out unnamed runtime values.
- Subrange selectors use their declared base for constant/range patterns. These
  tests do not construct a new domain value. Coverage may prove the complete
  declared interval; otherwise a fallback is required. Unreachable/out-of-domain
  patterns are errors, and an annotation on a whole-value binding names the
  subrange type, not its primitive base.
- Reference-type matching must account for null when the selector is nullable
  and otherwise requires a fallback unless it is a closed language variant.
- Guards do not prove coverage. Unreachable arms and duplicate constant patterns
  are errors. A catch-all after already exhaustive cases is unreachable.

An `is T as Name` pattern binds `Name` only inside its arm/guard. `Value is T`
outside patterns is a Boolean type test without introducing a binding.

Coverage preserves relationships between payload columns: all four combinations
of two bool payloads cover a case, while any missing combination leaves a gap.
Analysis has a bounded work limit and requests simpler nested patterns if exceeded.
When possible, `FP3025` names one missing pattern, such as
`Pair.Make(true, false)`. The pattern covers values left out by all unguarded
arms. Add the appropriate arm and check again for any remaining cases. Open
selectors may require a fallback even after the listed patterns are covered.
The selector's declared type governs coverage even when flow knows it non-null.
If foreign code violates the static assumptions and no arm matches, execution
throws InvalidOperationException rather than inventing a payload.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

## Source files

- [Binder.Cases.cs](../../../../src/Fpas.Compiler/Binding/Bodies/Binder.Cases.cs)
- [PatternCoverage.cs](../../../../src/Fpas.Compiler/Binding/Flow/PatternCoverage.cs)
- [CaseCoverageTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/CaseCoverageTests.cs)
- [CaseExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/CaseExecutionTests.cs)
