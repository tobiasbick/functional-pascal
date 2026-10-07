# AP11.1: Order-independent type declarations

Package: [AP11: Individual declarations](README.md)

Status: implemented on `codex/syntax-changes-2`. Current behavior is documented
in [type declaration order](../../../pascal/language/types/declaration-order.md).

## Scope

Resolve type references across all type declarations of a unit or program
regardless of order, including mutually recursive types (Q07), enum variants,
and record method signatures. Constants and variables keep declaration order.

## Prerequisites

None.

## Implementation

- Sema: collect type names, resolve structural type definitions, and register
  enum variants and record method signatures before checking ordered value
  declarations and executable bodies.
- Separate structural resolution from field-default and method-body checking.
  Check these expressions and bodies at the original type declaration position
  with the ordinary lexical scope and preceding value declarations available.
- Reject alias-only cycles and recursive definitions with no finite
  construction, using the rules below and a diagnostic naming the cycle.
- Keep value initialization order unchanged; do not reorder effects.
- Preserve scope and visibility of exported types across compiled units.

## Type and member visibility

Agreed: a type, its enum variants, and its record method signatures are
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

Agreed: nominal records and enums may be mutually recursive when their stored
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

The following examples are implemented acceptance cases. A runnable version is
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
    return record
      Members := [];
    end;
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
  var Group: Team := CreateTeam();
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

## Affected areas

- `crates/fpas-sema/src/check/entry.rs`, `check/decl/types/`.
- Structural collection and finite-construction checking live in
  `check/decl/types/collection/`; record fields, defaults, and ordered bodies
  have separate modules under `check/decl/types/records/`.
- Type/member name resolution and cycle-safe consumers of recursive types.
- Unit interface export of recursive types.

## Migration

None; existing order-dependent sources stay valid.

## Documentation

- `docs/pascal/language/types/README.md`, records/enums pages, and record
  methods: type/member availability, finite recursive construction, and
  declaration-order checking of value expressions.

## Verification

- Forward references in type bodies, routine signatures, and earlier value
  declarations; later enum variants and record method signatures are usable.
- Mutually recursive records through arrays and `Option`; recursive enums
  with a terminating alternative, including indirect terminating paths.
- Reject alias-only cycles, mandatory direct and mutual record-field cycles,
  and enums whose alternatives all necessarily continue the recursion.
  Diagnostics identify the participating types and fields/payloads.
- Existing recursive types, aliases to valid recursive types, imported
  recursive types, generic routine inference, and member visibility remain
  valid under the same rules.
- Field defaults and method bodies can use preceding constants and variables;
  references to later values remain errors. Retain existing free-routine and
  initializer ordering errors for values.
- Verify that type collection does not reorder effectful value initialization.
- Preserve qualification and ambiguity diagnostics for enum variants and
  record members, including references before their type declaration.

Regression coverage includes sema type-order tests, encoded-interface tests,
compiler runtime tests, compiled-unit reuse through the CLI, and
`tests/runner/type_order_test.fpas` in the repository suite.
