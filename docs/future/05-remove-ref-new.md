# Remove: Reference Types (`ref T`, `new`)

> Priority: 5 — depends on removing interfaces first (interfaces use `ref`
> for dynamic dispatch internally).

## What to remove

- `ref T` type constructor.
- `new T with ... end` allocation expression.
- Implicit dereference for `ref` values.
- Mutability-through-ref semantics.
- Keywords: `ref`, `new`.

## Current state

No example uses `ref` or `new`. TUI apps work with value-type records,
arrays, and primitives. No linked lists, trees, or shared mutable graphs
are needed.

## Scope

- **Lexer/token:** remove `ref` and `new` keywords.
- **Parser:** remove `ref` from `type_expr`, remove `new_expr` production.
- **Sema:** remove ref type checking, implicit dereference logic.
- **Compiler:** remove ref allocation, dereference bytecode.
- **Bytecode/VM:** remove or ignore ref-related opcodes (if dedicated ops
  exist).
- **Grammar (EBNF):** remove `ref` from `type_expr`, remove `new_expr`
  from `primary_expr`, remove `ref` and `new` from keyword list.
- **Docs:** remove ref sections from `02-basics.md` and `05-types.md`.
