# Generic routines and constraints

Generic routines preserve CLR type parameters and validate their constraints.

```pascal fragment
function Identity<T>(Value: T): T;
begin
  return Value;
end function;

function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin
  return Left.CompareTo(Right);
end function;
```

Generic arguments use `<...>` in declarations, types, and calls:
`Identity<int>(42)`. Calls may infer routine type arguments from supplied argument
types. Expected result type alone does not infer routine type arguments. Union
constructor inference also uses expected type as defined in [generic data types](../types/generics.md).
Namespace-qualified CLR generics are supported. General paths with type arguments
on every enclosing/nested segment remain a tracked
[baseline gap](../../../future/language-gaps.md#finish-the-existing-baseline-first).

Each `where T: ...;` follows the routine header or the type's `=` and precedes
the body/type definition. Constraints may include a base class, interfaces,
`class`, `class?`, `struct`, or `new()`. They follow CLR/C# combination rules,
with `new()` last. A record constraint example is:

```pascal fragment
type Box<T> =
where T: class;
record
  Value: T;
end record;
```

`new T()` requires an appropriate constructor constraint. There are no special
`Numeric`, `Printable`, `Comparable`, or `Error` constraints. Use actual .NET
interfaces and conversions. Instance-interface constraints work. Operators through
constrained static interfaces require CLR static-interface dispatch and remain
tracked baseline work. Unsupported requirements are diagnosed, never erased into
unchecked object operations. This does not add source interfaces or operators.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
