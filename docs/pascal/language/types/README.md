# Types

Composite and built-in type forms: records, enums, arrays, dictionaries, aliases, and generic routines.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_block`, `type_def`, `type_expr`, `record_type`, `enum_type`).

Each declaration repeats its keyword, including exported declarations such as
`public type Point = record ... end record;`. Grouped declaration sections are
invalid.

All type headers in a program or unit are collected before type definitions and
routine bodies are checked. A type reference may therefore name a later type in
the same program or unit. Recursive records and enums retain their nominal
identity; a cycle made only from type aliases is invalid. Constant and variable
initializers, including record field defaults, still follow declaration order:
type collection does not make later values available to an earlier initializer.

Generic records and enums use `of (...)` parameters and explicit applications.
Records and payload enums require a finite representable value: collections,
Option or an enum base variant may terminate recursion; mandatory cycles cannot.

| Topic | Description |
|-------|-------------|
| [Records](records.md) | Declaration, literals, fields, immutability, default values |
| [Record methods](record-methods.md) | Instance methods with implicit `Self`; bound method values; static functions and procedures via the type |
| [Record properties](record-properties.md) | Computed properties backed by instance `read` / `write` accessors |
| [Record events](record-events.md) | Single-handler events with `nil`, `Assigned`, and owner-only raise |
| [Record update](record-update.md) | `with` copy-and-override expressions |
| [Result and Option types](result-option-types.md) | `Result of (T, E)` and `Option of (T)` type forms |
| [Enumerations](enums.md) | Plain, backed, and data-carrying enums |
| [Arrays](arrays.md) | `array of (T)`, indexing, mutation |
| [Channels](channels.md) | `channel of (T)`, bounded FIFO communication and closure |
| [Task handles](../concurrency/task-handles.md#typed-task-handles) | `task` and `task of (T)`, handles whose `Wait` yields `T` |
| [Dictionaries](dictionaries.md) | `dict of (K, V)` |
| [Type aliases](type-aliases.md) | Semantic names for existing types |
| [Generics](generics.md) | Type parameters on records, enums, routines and record methods |

## See also

- [Error handling](../error-handling/README.md) — `try`, `panic`, combinators for `Result` / `Option`
- [Pattern matching](../pattern-matching/README.md) — enum and `Result` / `Option` `case` arms
- [Basics](../basics/README.md) — primitive types and operators
