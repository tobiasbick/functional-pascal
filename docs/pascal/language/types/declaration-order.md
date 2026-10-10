# Type declaration order

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
declarations. `Team` and `Member` may refer to each other.

Every type declaration starts with its own `type` keyword; in a unit, repeat
`public type` for every exported type. Record fields and enum members keep
their existing syntax inside the type body.

## Values and executable bodies

Constants, variables, and free routines retain declaration-order visibility.
Field defaults and record method bodies are checked at the type declaration's
original source position, with preceding values and routines in scope.
Parameters and local bindings keep their ordinary lexical scopes.

In the example, the `Status` default may reference `InitialState` because it
precedes `Team`. Moving `InitialState` below `Team` makes that reference invalid.
A later variable likewise remains unavailable in an earlier method body.
Collecting types does not reorder value initialization or its effects. Field
defaults are substituted when a record is constructed.

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
