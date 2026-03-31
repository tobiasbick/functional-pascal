# Remove: User-Defined Generic Records, Enums, and Type Aliases

> Priority: 7 — after interfaces are gone, simplify generic type support.

## What to remove

- Generic record declarations: `Pair<A, B> = record ... end;`
- Generic enum declarations: `Maybe<T> = enum ... end;`
- Generic type alias declarations: `IntBox = Box of integer;`
- The `of` keyword for type argument application on user-defined types.

## What stays

- **Generic functions:** `function Identity<T>(Value: T): T` — useful for
  stdlib and utility code.
- **Built-in `Result of T, E` and `Option of T`** — compiler-managed generic
  types, not user-defined.
- **Constraints:** `Comparable`, `Numeric`, `Printable` — useful on generic
  functions.

## Scope

- **Parser:** remove `type_params` from `type_def` for records, enums, and
  aliases. Keep `type_params` on `function_heading` / `procedure_heading`.
- **Sema:** remove generic type instantiation for user types. Keep generic
  function instantiation.
- **Compiler:** remove monomorphization / type-erasure for user generic types.
- **Grammar (EBNF):** remove `type_params` from `type_def`.
- **Docs:** simplify `05-types.md` generics section. Update `04-functions.md`
  to clarify only functions support type parameters.
