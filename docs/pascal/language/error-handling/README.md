# Error handling

Structured error handling with `Result` and `Option` for expected failures, and `panic` for unrecoverable errors.

| Purpose | Form |
|---------|------|
| Propagate returned error or absence | `try Expression` in a compatible function |
| Test a variant without binding data | `Value is Ok(_)`, `is Error(_)`, `is Some(_)`, or `is None` |
| Bind payloads, handle locally, or select a plain default | Exhaustive `case` |
| Abort deliberately on Error or None | `Value.Unwrap()` |
| Transform wrapped success | `Value.Map(F)` |
| Compose a fallible/optional operation | `Value.AndThen(F)` |
| Recover within a wrapper | `Value.OrElse(F)` |

The four [Std.Parse](../../std/text/parse.md) text parsers return Result values.
Formatting and conversion of typed values belong to [Std.Conv](../../std/text/conv.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` for `Result` and `Option`, `primary_atom` for `try`, `panic_stmt`, `destructure_pattern`).

| Topic | Description |
|-------|-------------|
| [Result](result.md) | `Result of (T, E)`, `Ok`, `Error` |
| [Native error families](../types/errors.md) | Nominal error payloads, typed causes and automatic output |
| [Option](option.md) | `Option of T`, `Some`, `None` |
| [Try operator](try.md) | Early propagation with `try` |
| [Combinators](combinators.md) | `Map`, `AndThen`, `OrElse` overview |
| [Panic](panic.md) | Unrecoverable abort |

Type forms: [Result and Option types](../types/result-option-types.md). Pattern matching: [Result and Option patterns](../pattern-matching/result-option-patterns.md).

## Keywords

`Result`, `Option`, `Ok`, `Error`, `Some`, `None`, `try`, `panic` are reserved keywords.

## See also

- [`Result operations`](../types/result-operations.md), [`Option operations`](../types/option-operations.md)
