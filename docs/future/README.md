# Future Features

Planned changes for Functional Pascal, in execution order.

## Removals (simplification)

| # | Feature | Description |
|---|---------|-------------|
| 1 | ~~[`forward`](01-remove-forward.md)~~ | **Done** |
| 2 | ~~[`Compiler Directives`](02-remove-compiler-directives.md)~~ | **Done** |
| 3 | ~~[Inline Lambdas](03-remove-inline-lambdas.md)~~ | **Done** |
| 4 | [Nested Patterns](04-remove-nested-patterns.md) | Remove deep destructuring and `_` wildcard — simple matching stays |
| 5 | [`ref` / `new`](05-remove-ref-new.md) | Remove reference types and heap allocation |
| 6 | [Interfaces](06-remove-interfaces.md) | Remove `interface`, `implements`, `extends`, virtual dispatch |
| 7 | [Generic Types](07-remove-generic-types.md) | Remove user-defined generic records, enums, type aliases — generic functions stay |
| 8 | [Channels + Select](08-remove-channels-select.md) | Remove `channel`, `select`, `Std.Channel` — `go`/`task`/`Wait` stays |
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |

## Additions

| # | Feature | Description |
|---|---------|-------------|
| 10 | [String Format](10-add-string-format.md) | `Format()` function in `Std.Str` |
| 11 | [Extended Colors](11-add-extended-colors.md) | 256-color and truecolor in `Std.Console` |
| 12 | [String Repeat](12-add-string-repeat.md) | `Repeat()` function in `Std.Str` |

## Not yet planned

| Feature | Description |
|---------|-------------|
| [Libraries](libraries.md) | Project kind `library`, export rules |
