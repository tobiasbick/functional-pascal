# Declaration order

Types declared in the same program or unit are visible throughout that program
or unit, regardless of declaration order. This includes aliases, enum variants,
and record member signatures. Qualified names and existing visibility and
short-name ambiguity rules apply at every source position.

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
    return Team(Members := []);
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
declarations. `Team` and `Member` may refer to each other.

Every type declaration starts with its own `type` keyword; in a unit, repeat
`public type` for every exported type. Record fields and enum members keep
their existing syntax inside the type body.

## Named routines

Named function and procedure signatures are visible throughout their lexical
declaration scope. The same rule applies to free routines, record methods,
and nested named helpers. Calls may precede declarations, and sibling routines
may call each other recursively. No `forward` declaration is needed.

The nearest lexical scope determines the selected declaration. An inner named
routine shadows an outer name throughout its scope, including in an earlier
sibling body. Duplicate names in one scope are errors. Imports and member
visibility retain their ordinary boundaries.

## Values and initialization

Values initialize in source order. A routine body may refer to later values
in its enclosing lexical environment, but a use is accepted only when the
compiler can prove that the bindings it may access are initialized. This covers
direct and transitive calls, callbacks, captures, and omitted record defaults.
An ordinary expression cannot read or write a binding before initialization.
Initializers are never reordered to satisfy dependencies.

```pascal
const Initial: integer := Twice(3);

function Twice(Value: integer): integer;
begin
  return Value * 2;
end function;
```

This initializer is valid because `Twice` uses only its argument. If a helper
reads a later binding, call it after that binding has initialized, or pass
already initialized data as an explicit argument. Unsafe or unprovable uses
produce `FP3041`; unknown callable dependencies do not imply safety.

`const` and `var` follow the same availability rule, including callable values
stored in bindings. An anonymous function is a value initialized where its
expression is evaluated, rather than a scope-wide named routine signature.
Creating a capturing callable requires its captured bindings to be available.
Existing value snapshots, shared mutable cells, and reference lifetimes remain
in force.

[Subrange bounds](subranges.md) use compile-time values visible at the type
declaration's source position. They may reference preceding constants and
imported static constants; a later local constant is unavailable.

Field defaults retain the record's declaring lexical environment and run when
the record is constructed. Their dependencies must be initialized at that
construction point. Supplying a field explicitly does not evaluate its default.
In the example, `Status` may reference `InitialState` even if that binding is
declared below `Team`, provided construction happens after initialization.
Collecting types or routine signatures does not reorder initialization or effects.
Dependency units initialize before consumers; compiled interfaces retain
callback dependencies needed by the same checks.

## Finite recursive values

Records and enums may be mutually recursive when a finite value can be
constructed. The check follows stored fields and enum payloads across types.

Shared finite type definitions are valid when referenced by multiple required
fields. The compiler checks one shared construction graph and reuses its results
across declarations; shared paths do not repeatedly expand the same definitions.
Terminating alternatives are evaluated across the complete graph, independently
of declaration or variant order.

- Every required record field must admit a finite value. Mandatory direct or
  mutual record cycles are rejected.
- `Option` can terminate with `None`. Arrays and dictionaries can terminate
  with an empty container; channels store a handle rather than inline elements.
- An enum needs an alternative whose required payload fields all admit finite
  construction. The terminating path may pass through other types.
- `Result` can terminate through either its success or error payload.
- Callable signatures, record method signatures, and task output types do not
  inline their referenced values.
- Alias chains must reach a concrete type. Alias-only cycles are rejected,
  including cycles wrapped in container types. Aliases to finite recursive
  records and enums remain valid.

```pascal
type Chain = enum
  Empty;
  Link(Next: Chain);
end enum;
```

`Chain.Empty` terminates recursion. In contrast, `Next: Node` in a record
`Node` necessarily repeats the same required field and is rejected. Use
`Next: Option of Node` to provide a terminating value.

Alias cycles produce `FP3023`; mandatory recursive values with no terminating
path produce `FP3024`, naming the participating types and fields or payloads.
Recursive public types retain their identities and member visibility across
compiled-unit interfaces.

Generic records and enums follow these rules after type substitution. On every
reference in a recursive generic cycle, pass the declaring parameters unchanged
and in the same positions. Different parameter names in mutually recursive
declarations are allowed; wrapping, replacing, or reordering parameters is an
error. See [generic data types](generics.md).

## See also

- [Records](records.md)
- [Enumerations](enums.md)
- [Record methods](record-methods.md)
- [Type aliases](type-aliases.md)
