# Error handling

Structured error handling with `Result` and `Option` for expected failures, and `panic` for unrecoverable errors.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` for `Result` and `Option`, `primary_atom` for `try`, `panic_stmt`, `pattern`).

| Topic | Description |
|-------|-------------|
| [Result](result.md) | `Result of (T, E)`, `Result.Ok`, `Result.Error` |
| [Option](option.md) | `Option of (T)`, `Option.Some`, `Option.None` |
| [Try operator](try.md) | Early propagation with `try` |
| [Combinators](combinators.md) | `Map`, `AndThen`, `OrElse` overview |
| [Panic](panic.md) | Unrecoverable abort |

Type forms: [Result and Option types](../types/result-option-types.md). Pattern matching: [Result and Option patterns](../pattern-matching/result-option-patterns.md).

## Keywords

`Result`, `Option`, `Ok`, `Error`, `Some`, `None`, `try`, and `panic` are reserved
keywords. Constructor spellings combine the owner and member, such as
`Result.Ok(Value)` and `Option.None`.

## See also

- [`Std.Results`](../../std/result/result.md), [`Std.Options`](../../std/result/option.md)
