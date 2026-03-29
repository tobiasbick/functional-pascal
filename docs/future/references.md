# Reference Types

Implemented in the current language.

Use the stable language reference for the canonical specification:

- [../pascal/05-types.md](../pascal/05-types.md) for `ref T`, `new T with ... end`, recursive records, and implicit dereference.
- [../pascal/02-basics.md](../pascal/02-basics.md) for assignment and mutability semantics.
- [../specs/grammar.ebnf](../specs/grammar.ebnf) for the formal grammar.

`ref T` creates a shared reference type for heap-allocated records. `new T with ... end` allocates a record and returns `ref T`. Field access, field assignment, indexing, and method calls dereference `ref` values automatically.
