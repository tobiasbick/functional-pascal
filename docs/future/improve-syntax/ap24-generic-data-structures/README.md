# AP24: Generic data structures

Status: agreed direction. Effort: very large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Generic records and enums work across concrete types with explainable rules,
including recursion and nested matching, rather than per-type exceptions, and
every type application uses the same `of` form.

## Decisions

- Every type application uses `of`. One type argument is written without
  parentheses; several are always parenthesized: `array of string`,
  `Option of User`, `Result of (User, LoadError)`,
  `Pair of (Lookup of string, integer)`. `dict of K to V` keeps its form.
- User-defined generic types declare their parameters with the same word:
  `type Lookup of T = enum`, `type Pair of (K: Comparable, V) = record`.
  Routines keep `function Max<T: Comparable>(...)`, because routine type
  arguments are always inferred and never written at a use site.
- Angle-bracket applications such as `Lookup<string>` are errors whose
  diagnostic shows the `of` form.
- Constructor type arguments are inferred from constructor arguments,
  otherwise from the expected type; if neither determines a parameter
  (`Lookup.Missing` without context), an annotation is required.
- Constraints start from the existing routine constraints, not a broad
  typeclass model.

```pascal
type Lookup of T = enum
  Found(Value: T);
  Missing;
  Failed(Message: string);
end enum;

type Pair of (K: Comparable, V) = record
  Key: K;
  Value: V;
end record;

const L: Lookup of string := Lookup.Found('x');
const R: Result of (User, LoadError) := Load(Id);
```

## Dependencies

- AP10 (typed construction for generic records).
- AP20 (nested patterns for generic enums).

AP28 is transferred after this package.

## Order

AP24.1 changes the built-in application form and is independent of user
generics. AP24.2 adds generic records, AP24.3 generic enums with recursion.

## Work packages

- [ ] [AP24.1: Parenthesized type argument lists](01-parenthesized-type-arguments.md)
- [ ] [AP24.2: Generic records](02-generic-records.md)
- [ ] [AP24.3: Generic enums and recursion](03-generic-enums.md)

## Acceptance

Generic records and enums work across concrete types with explainable rules,
including recursion and nested matching, rather than per-type exceptions, and
every type application uses the same `of` form.

## Reference

The reference branch `codex/syntax-changes` found that
`fpas-parser/src/parser/decl/data/type_defs.rs` explicitly rejects generic type
definitions and that sema currently rejects coercion of a generic function
value to a concrete signature.
