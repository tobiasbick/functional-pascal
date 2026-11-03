# Casts and runtime types

`cast<T>(Expression)` performs an explicit numeric or CLR reference conversion.
Numeric casts are checked unless enclosed by `unchecked`. An incompatible
reference cast throws `InvalidCastException`; it is not an optional conversion.
`Value is T` tests runtime type compatibility, yielding false for null.
`typeof(T)` produces `System.Type`. Casts do not parse strings or translate
application-defined union values into nullable values.
Compiler-defined casts wrap/unwrap [distinct types and subranges](../types/data-domains.md).
Subrange bounds remain checked inside unchecked expressions.

`typeof(T)` uses a statically resolved type, not a value expression. It produces
a non-null Type without constructing an instance or running the operand type's
static constructor/unit initializer. Transparent aliases identify their CLR target;
nominal source types retain their emitted identity. Reference nullable annotations
do not create a distinct runtime type, while nullable values retain Nullable<T>
identity. Closed generic types and in-scope type parameters follow the existing
type/constraint rules. See the [implemented typeof milestone](../../../implementation/typeof.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
