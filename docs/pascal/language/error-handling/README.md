# Error handling

Structured error handling with `Result` and `Option` for expected failures, and `panic` for unrecoverable errors.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`result_type`, `option_type`, `try` expression, `panic_stmt`, `destructure_label`).

| Topic | Description |
|-------|-------------|
| [Result](result.md) | `Result of T, E`, `Ok`, `Error` |
| [Option](option.md) | `Option of T`, `Some`, `None` |
| [Try operator](try.md) | Early propagation with `try` |
| [Combinators](combinators.md) | `Map`, `AndThen`, `OrElse` overview |
| [Panic](panic.md) | Unrecoverable abort |

Type forms: [Result and Option types](../types/result-option-types.md). Pattern matching: [Result and Option patterns](../pattern-matching/result-option-patterns.md).

## Keywords

`Result`, `Option`, `Ok`, `Error`, `Some`, `None`, `try`, `panic` are reserved keywords.

## See also

- [`Std.Result`](../../std/result/result.md), [`Std.Option`](../../std/result/option.md)
