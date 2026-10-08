# AP16.2: Migrate immutable var to const

Package: [AP16: Immutable and mutable bindings](README.md)

Status: complete.

## Result

Repository sources, embedded fixtures, generators, editor examples and current
documentation use `const` for immutable bindings. Reassignable storage uses
`var`; intrinsic declarations export constants as `public const`.

Initialization order, reachability, captures, member names and shadowing retain
their behavior. Scalar guard-binding regressions cover fresh arm names and
resolved uses when an outer declaration would otherwise become a value label.

## Regression coverage

Compiler, CLI and FPAS tests cover initializers, shared captures, nested
shadowing, guard bindings and same-named fields. Authoring guidance and snippets
prefer `const` when reassignment is unnecessary.
See [variables](../../../pascal/language/basics/variables.md).
