# Type names and aliases

A transparent alias gives an existing type a source name without creating a
new CLR identity. Use nominal domains when a distinct identity is required.

```pascal fragment context=declarations
type UserName = string;
type Names = Collections.List<string>;
type Matrix = array[,] of double;
```

`type Name = T;` is a transparent source alias and does not emit a new CLR type.
It may not be recursive. Type arguments uniformly use `<...>`:
`List<string>`, `Task<string>`, and `Dictionary<string, int>`.
Nested generic closers such as `List<List<int>>` are valid.
Type names and generic arity are resolved exactly. Source declarations cannot
differ solely by capitalization accidentally: capitalization denotes distinct
names and is never folded by import lookup.
Public aliases cannot expose private types. Their target signature and nested
nullable annotations survive cross-assembly FPAS use through the versioned alias
metadata contract; other .NET consumers see the target types in ordinary signatures.

Use `type UserId = distinct long;` for a nominal scalar identity and
`type Percent = int range 0..100;` for a checked integral subrange. Both have
defined CLR representations and conversions; see [data domains](data-domains.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
