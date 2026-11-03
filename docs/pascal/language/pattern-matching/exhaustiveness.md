# Exhaustiveness and reachability

- Language unions require exhaustive coverage in both
  statement and expression forms. Coverage descends into nested union patterns.
- `bool` requires true and false, or a fallback.
- Integer, string, and CLR enum selectors require an unguarded catch-all pattern
  or `else`. Enum metadata does not rule out unnamed runtime values.
- Subrange selectors use their declared base for constant/range patterns. These
  tests do not construct a new domain value. Coverage may prove the complete
  declared interval; otherwise a fallback is required. Unreachable/out-of-domain
  patterns are errors, and an annotation on a whole-value binding names the
  subrange type, not its primitive base.
- Reference-type matching must account for null when the selector is nullable
  and otherwise requires a fallback unless it is a closed language union.
- Guards do not prove coverage. Unreachable arms and duplicate constant patterns
  are errors. A catch-all after already exhaustive cases is unreachable.

An `is T as Name` pattern binds `Name` only inside its arm/guard. `Value is T`
outside patterns is a Boolean type test without introducing a binding.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
