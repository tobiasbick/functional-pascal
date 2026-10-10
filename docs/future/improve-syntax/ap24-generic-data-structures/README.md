# AP24: Generic data structures

Status: complete (AP24.1, AP24.2, AP24.3). Effort: very large.
Completion is tracked in the
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
- Constructors use the ordinary type or variant name, such as
  `Pair(Key := 1, Value := 'x')` or `Lookup.Found('x')`. Explicit type
  arguments appear only in type declarations and annotations; constructors
  do not accept an `of` application before their argument list.
- Constructor type arguments follow the
  [inference rules below](#constructor-type-inference). Field defaults never
  contribute to inference.
- Constraints start from the existing routine constraints, not a broad
  typeclass model.
- Generic field defaults must be valid for every type permitted by the
  parameter constraints. Recursive generic references pass their type
  parameters unchanged and in the same positions.

AP24.1 implements the built-in application spelling; AP24.2 implements
user-defined generic records; AP24.3 implements generic enums. The examples
below use the implemented declaration, application, and constructor forms.

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
const P: Pair of (integer, string) := Pair(Key := 1, Value := 'x');
const R: Result of (User, LoadError) := Load(Id);
```

### Constructor type inference

Consider all explicitly supplied field values together when determining type
arguments. Named-field order does not affect inference. The expected type
fills in parameters that the supplied values leave undetermined. Supplied
values and the expected type must agree on every determined parameter.

Conflicting evidence is an error. Do not search for a common type or promote
numeric types to resolve a conflict. If a parameter remains undetermined, the
diagnostic requires a type annotation. For example, `Lookup.Missing` needs an
expected `Lookup of T`; `const L: Lookup of string := Lookup.Missing;`
determines `T` as `string`.

This inference applies to constructor type arguments. It does not broaden
AP22's rules for local binding inference. Evaluation retains AP10 and AP09
written-order rules even though inference considers the fields together.

### Generic record defaults

Check defaults at the generic record declaration against the declared
parameter constraints. A default must be valid for every type those
constraints permit; a later instantiation cannot make an invalid generic
default valid. Omitted defaults supply no evidence for constructor type
inference.

For an unconstrained `T`, `Value: T := 0` is invalid. A field declared as
`Value: Option of T := None` is valid, but omitting that field does not
determine `T`. Default visibility, declaration scope, and evaluation order
retain AP10's rules.

### Recursive type arguments

On every reference in a recursive generic type cycle, forward the declaring
type's parameters unchanged and in the same positions. Parameter names may
differ between mutually recursive declarations. Reordering, replacing, or
wrapping parameters in another type is rejected: `Tree of T` may refer to
`Tree of T`, but not recursively to `Tree of (array of T)`.

After this check, apply AP11.1's finite-construction rules. Unchanged type
arguments alone do not make a mandatory stored-value cycle valid; recursion
still needs a terminating construction.

## Open decisions

None for the rules above. AP24.1 retains its agreed type-application spelling;
AP24.2 and AP24.3 implement the constructor, default, and recursion rules.

## Dependencies

- AP10 (typed construction for generic records).
- AP20 (nested patterns for generic enums).
- AP03.2 (closed-enum exhaustiveness for AP24.3).
- AP11.1 (finite construction of recursive records and enums).

AP28's tooling scope is transferred to
[editor/LSP planning](../../editor-structured-editing.md).

## Order

AP24.1 changes the built-in application form and is independent of user
generics. AP24.2 adds generic records, AP24.3 generic enums with recursion.

## Work packages

- [x] [AP24.1: Parenthesized type argument lists](01-parenthesized-type-arguments.md)
- [x] [AP24.2: Generic records](02-generic-records.md)
- [x] [AP24.3: Generic enums and recursion](03-generic-enums.md)

## Acceptance

Generic records and enums work across concrete types with explainable rules,
including recursion and nested matching, rather than per-type exceptions, and
every type application uses the same `of` form.

Constructor inference is independent of named-field order and reports
conflicting or undetermined parameters. Constructors use the ordinary type
or variant name; explicit type arguments stay in type declarations and
annotations. Defaults are checked against the declared constraints and do
not determine type arguments. Recursive generic references preserve parameter
positions and admit finite construction.
