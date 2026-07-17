# Types

Composite and built-in type forms: records, enums, arrays, dictionaries, aliases, and generic routines.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_decl`, `type_expr`, `record_type`, `enum_type`).

| Topic | Description |
|-------|-------------|
| [Records](records.md) | Declaration, literals, fields, immutability, default values |
| [Record methods](record-methods.md) | Instance methods with implicit `Self`; bound method values; static functions via the type |
| [Record properties](record-properties.md) | Computed properties backed by instance `read` / `write` accessors |
| [Record events](record-events.md) | Single-handler events with `nil`, `Assigned`, and owner-only raise |
| [Record update](record-update.md) | `with` copy-and-override expressions |
| [Result and Option types](result-option-types.md) | `Result of T, E` and `Option of T` type forms |
| [Enumerations](enums.md) | Plain, backed, and data-carrying enums |
| [Arrays](arrays.md) | `array of T`, indexing, mutation |
| [Dictionaries](dictionaries.md) | `dict of K to V` |
| [Type aliases](type-aliases.md) | Semantic names for existing types |
| [Generics](generics.md) | Type parameters on routines and record methods |

## See also

- [Error handling](../error-handling/README.md) — `try`, `panic`, combinators for `Result` / `Option`
- [Pattern matching](../pattern-matching/README.md) — enum and `Result` / `Option` `case` arms
- [Basics](../basics/README.md) — primitive types and operators
