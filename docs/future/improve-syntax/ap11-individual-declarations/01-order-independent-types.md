# AP11.1: Order-independent type declarations

Package: [AP11: Individual declarations](README.md)

Status: complete.

## Result

Type names, structural definitions, enum variants and record method signatures
are available throughout their unit or program. Value declarations and their
initialization effects retain source order. Structural collection and finite
construction checking live in `crates/fpas-sema/src/check/decl/types/collection/`;
record defaults and ordered bodies have separate checking paths.

## Type and member visibility

A type, its enum variants, and its record method signatures are
available throughout their declaring unit or program, including in earlier
value initializers and routine bodies. For example, `State.Ready` and
`Team.Empty()` may appear before the declarations of `State` and `Team`.
Aliases to valid types follow the same order-independent resolution.

Existing qualification, case-insensitivity, short-name ambiguity, and member
visibility rules still apply. This is whole-unit type resolution; it does not
introduce local type declarations, make unrelated routines order-independent,
or permit import cycles between units.

Field defaults and record method bodies are checked at their type definition's
original source position. Parameters and local bindings keep their ordinary
scopes. References to constants and variables require those values to have
already been declared; later values do not become visible through type
collection. Free routine bodies and value initializers retain their existing
declaration-order rules for values.

## Recursive types

Nominal records and enums may be mutually recursive when their stored
values admit a finite construction. The check follows stored fields and enum
payloads across type references, rather than rejecting every dependency cycle.

- Reject cycles consisting only of aliases, such as `A = B` and `B = A`.
  An alias to a valid recursive record or enum remains valid.
- Allow optional or empty-container paths that can end the recursion. For
  example, `Next: Option of Node` can hold `None`, and
  `Children: array of Node` can hold an empty array.
- Reject mandatory record-field cycles with no finite construction, including
  direct `Next: Node` self-reference and mutually mandatory `A`/`B` fields.
- A recursive enum needs an alternative whose payloads allow the recursion to
  end, directly or through other types. `Empty` alongside `Link(Next: Chain)`
  is valid. If every alternative necessarily continues the recursion, reject
  the cycle. All required payload fields of the terminating alternative must
  themselves admit finite construction.
- Method signatures are type information, not stored fields. A method
  accepting or returning its record type does not itself require a recursive
  stored value.
- Preserve existing valid recursive types, including their use through
  routine signatures, generic routine inference, and compiled-unit interfaces.

## Examples

The examples cover type visibility and finite recursive construction. A runnable version is
in [type_order.fpas](../../../../examples/pascal/records/type_order.fpas).

```pascal
program TypeOrderExample;

const InitialState: State := State.Ready;

function CreateTeam(): Team;
begin
  return Team.Empty();
end function;

type Team = record
  Members: array of Member;
  Status: State := InitialState;

  static function Empty(): Team;
  begin
    return Team(
      Members := []
    );
  end function;
end record;

type Member = record
  Home: option of Team;
end record;

type State = enum
  Ready;
  Busy;
end enum;

begin
  const Group: Team := CreateTeam();
  discard Group;
end.
```

`State.Ready`, `Team`, and `Team.Empty()` are available before their type
declarations. The `Team`/`Member` recursion can end in an empty member array or
`None`. The `Status` default may use `InitialState` because that value was
declared before `Team`.

A terminating recursive enum is also valid:

```pascal
type Chain = enum
  Empty;
  Link(Next: Chain);
end enum;
```

A mandatory self-field has no finite construction and is rejected:

```pascal
type Node = record
  Next: Node;
end record;
```

Type collection must not make this later value visible to the field default:

```pascal
type Settings = record
  Limit: integer := DefaultLimit;
end record;

const DefaultLimit: integer := 10;
```

## Regression coverage

Sema, interface, compiler, CLI and `tests/runner/type_order_test.fpas` tests
cover forward references, finite recursion, rejected cycles, visibility,
ambiguity, generics, source-order value checks and initialization effects.
See [type declaration order](../../../pascal/language/types/declaration-order.md).
