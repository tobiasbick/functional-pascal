# Overloads

Free routines and instance/static record methods may overload by CLR-compatible signatures.
Return type alone does not distinguish overloads. Parameter labels do not
distinguish signatures; static versus instance alone also does not distinguish
source overloads. `ref`, `in`, and `out` alone cannot distinguish overloads
whose underlying byref parameter types are identical.

Resolution uses argument count, names, explicit generic arguments, applicable
implicit conversions, and CLR/C# better-function-member rules. A method return
type alone never selects an overload. No applicable candidate or multiple best
candidates is a diagnostic listing the relevant signatures. Use explicit type
arguments or casts to resolve an ambiguity; do not choose by declaration order.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
