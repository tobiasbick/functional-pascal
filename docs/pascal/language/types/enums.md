# CLR enums

An enum names integral CLR constants. It is distinct from a payload-carrying variant.

```pascal fragment context=declarations
type Direction = enum : byte
  North = 0;
  East = 1;
  South = 2;
  West = 3;
end enum;
```

An enum is an actual CLR enum with underlying type `byte`, `sbyte`, `short`,
`ushort`, `int`, `uint`, `long`, or `ulong`, default `int`.
Explicit values must fit that type; omitted values increment the previous value,
starting at zero. Duplicate values are allowed for CLR aliases. Payloads are not
allowed. Use `Direction.North` and explicit casts to/from the underlying type.

CLR enum values may contain unnamed underlying values. Exhaustive variant rules do
not apply to enums. Same-type bitwise operations are supported; a `[FlagsAttribute]`
annotation documents intended flags use but does not restrict runtime values.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
