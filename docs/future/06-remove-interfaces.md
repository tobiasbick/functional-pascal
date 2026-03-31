# Remove: Interfaces

> Priority: 6 — larger removal, touches type system, records, and dispatch.

## What to remove

- `interface` type declarations.
- `implements` clause on records.
- `extends` for interface composition.
- Virtual dispatch (`CallVirtual` bytecode).
- Keywords: `interface`, `implements`, `extends`.

## Current state

No example uses interfaces. Enum-based dispatch (`case Mode of ...`) covers
all dispatch needs for TUI applications.

## Scope

- **Lexer/token:** remove `interface`, `implements`, `extends` keywords.
- **Parser:** remove `interface_type`, `implements_clause`,
  `interface_method_sig` productions.
- **Sema:** remove interface declaration checking, implements validation,
  method signature matching.
- **Compiler:** remove `CallVirtual` emission.
- **Bytecode/VM:** remove `CallVirtual` opcode.
- **Grammar (EBNF):** remove all interface-related productions, remove 3
  keywords.
- **Docs:** remove interface mentions from `05-types.md`. Remove
  `docs/future/interfaces.md` (already done).
