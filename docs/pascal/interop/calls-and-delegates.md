# CLR calls, generics, and delegates

Use `<...>` for explicit generic arguments, including static methods:

```pascal fragment
const Text: string := Json.JsonSerializer.Serialize<Player>(PlayerValue);
```

The compiler must check metadata constraints, accessibility, optional arguments,
`params`, named arguments, byref argument modes, and conversions before emission.
Selection is static. It does not fall back to runtime reflection or dynamic
invocation when overload resolution fails.

Compatible lambdas and named routines convert to CLR delegates. For instance,
`Items.Sort(function(A: int; B: int): int => B.CompareTo(A));` selects a compatible
Comparison<int> overload. A matching callable value is not automatically an
expression tree. Consumers requiring Expression<TDelegate> need a separately
specified feature.

Extension methods are callable in **static form** in the baseline. Import their
declaring type, for example System.Linq.Enumerable, and call
`Enumerable.Select<int, int>(Values, Transform)`. Automatic extension-method
discovery through dot syntax is not included. CLR instance methods remain normal
dot calls.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
