# CLR calls, generics, and delegates

Use `of T` or `of (A, B)` for explicit generic arguments, including static methods:

```pascal fragment context=json-player
const Text: string := Json.JsonSerializer.Serialize of Player(PlayerValue);
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
`Enumerable.Select of (int, int)(Values, Transform)`. Automatic extension-method
discovery through dot syntax is not included. CLR instance methods remain normal
dot calls.

## Generic owners and indexers

Members of a constructed CLR type substitute its actual arguments. Public nested
types retain their declaring-type identity. Source paths put each segment's own
arguments on that segment, such as `(N.Outer of string).Inner of int`; a nested type
with no parameters of its own uses `(N.Outer of string).Leaf`. Its CLR instantiation
includes the outer arguments followed by the inner arguments. Imported constraints
include class/class?, struct, new(), notnull, and base/interface requirements.
Reference variance follows CLR in/out flags. Invariant nullable arguments and
array element annotations cannot be weakened to permit unsafe writes.

Indexers resolve the CLR DefaultMember property and public getter/setter
separately. FPAS index expressions pass their arguments by value. Receiver,
indices, and assigned value execute once in written order.
Writable struct elements/receivers retain their storage; no synthetic collection
operation or reflection fallback replaces the selected accessor.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)

## Source files

- [json-player.fpas.txt](../../../apps/doc-examples/contexts/json-player.fpas.txt)
- [Binder.Calls.cs](../../../src/Fpas.Compiler/Binding/Bodies/Binder.Calls.cs)
- [ReferenceCatalog.Callables.cs](../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.Callables.cs)
