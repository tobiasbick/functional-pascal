# Distinct types

A distinct type gives a scalar value its own type identity. Values of different
distinct types cannot be mixed up, and a distinct value cannot be used as its
underlying type without an explicit conversion.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`distinct_type`, `distinct_conversion`).

```pascal
type UserId = distinct integer;

type OrderId = distinct integer;

type Name = distinct string;
```

A [type alias](type-aliases.md) such as `type Count = integer;` is only another
name for `integer`. A distinct type such as `UserId` is a new type: `UserId`,
`OrderId`, and `integer` are three incompatible types.

## Underlying types

The underlying type is `integer`, `real`, `string`, or `boolean`, written
directly or through an alias of one of them:

```pascal
type Raw = integer;

type AccountId = distinct Raw;
```

Other underlying types are errors. Records and enums already have their own
type identity, and arrays, dictionaries, channels, `Option`, `Result`, tasks,
function types, generic parameters, and other distinct types are not scalar.
Use a record for a structured domain value.

`type UserId = type integer;` is not a declaration form; write
`type UserId = distinct integer;`.

## Conversions

Construction and unwrapping are explicit. Call the distinct type to wrap a value
of its underlying type, and call the underlying type name to unwrap it:

```pascal
const Admin: UserId := UserId(1);
const Raw: integer := integer(Admin);
```

Each conversion takes exactly one positional value:

- `UserId(Value)` requires `Value` to have the underlying type `integer`, or to
  already be a `UserId`.
- `integer(Value)` requires `Value` to be a distinct value whose underlying type
  is exactly `integer`. An alias of the underlying type unwraps the same way,
  for example `Raw(Admin)`.
- A distinct value never converts directly to another distinct type. Unwrap it
  first: `OrderId(integer(Admin))`.
- Built-in type names only unwrap distinct values. `integer(3.5)` is an error;
  use conversion routines such as `Trunc`, `Round`, `IntToReal`, or `IntToStr`
  for built-in conversions.

Implicit conversion is an error in both directions, including assignments,
initializers, arguments, and return values:

```pascal
procedure Ban(Id: UserId);
begin
  WriteLn('banned ', integer(Id));
end procedure;

Ban(UserId(42));  // valid
Ban(42);          // error: expected `UserId`, found `integer`
Ban(OrderId(42)); // error: expected `UserId`, found `OrderId`
```

A conversion of a compile-time constant is itself a compile-time constant, so
`const Admin: UserId := UserId(1);` and `integer(Admin)` can be used wherever a
compile-time constant is required. A conversion of a computed value is computed.

Conversions have no runtime cost: a distinct value is stored as its underlying
value. The [debugger](../../tools/debugger.md) evaluates the same conversions.

## Comparisons

Two values of the same distinct type compare with `=`, `<>`, `<`, `>`, `<=`, and
`>=`, using the ordering of the underlying type:

```pascal
const First: UserId := UserId(1);
const Second: UserId := UserId(2);

const Earlier: boolean := First < Second;   // true
const Same: boolean := First = UserId(1);   // true
```

Comparing different distinct types, or a distinct value with its underlying type,
is an error: `First = OrderId(1)` and `First = 1` are rejected; convert one side
explicitly. Records and other values that contain distinct fields compare
structurally under the usual equality rules.

`in` follows equality. `Id in Ids` searches an `array of UserId`, and `Id in Owners`
tests the keys of a `dict of UserId to T`. A distinct type may be a dictionary
key. Substring tests are not inherited: for a `distinct string`, write
`string(Part) in string(Text)`.

A distinct type satisfies the `Comparable` [generic constraint](generics.md), so
`Max<T: Comparable>(First, Second)` returns a `UserId`.

## `case`

A distinct value can be a `case` selector when its underlying type is `integer`,
`string`, or `boolean`. Value labels and range endpoints are compile-time
constants of the same distinct type: a conversion of a constant or a `const` of
that type.

```pascal
const Admin: UserId := UserId(1);

case Id of
  when Admin:
    WriteLn('admin');
  when UserId(2), UserId(3):
    WriteLn('staff');
  when UserId(4)..UserId(9):
    WriteLn('early');
  when const Other if Other > UserId(100):
    WriteLn('large');
  else
    WriteLn('other');
end case;
```

A plain `1` label is a type mismatch, as is a label of another distinct type.
The same typed values work inside patterns, for example `when Some(UserId(1)):`
or `if Found is Some(Admin) then`. A `distinct real` is not a `case` selector.

## Other operations

A distinct type inherits no other operators, built-in type operations, or generic
constraints from its underlying type:

- Arithmetic, string concatenation, and logical operators are errors.
  `UserId(1) + 1` is rejected; unwrap explicitly, or use a record and functions
  for quantities that need arithmetic.
- Built-in dot operations are not available: `Name.Length()` on a
  `distinct string` is an error; write `string(Name).Length()`.
- Standard-library routines and console output do not unwrap distinct values:
  write `WriteLn(integer(Id))`, not `WriteLn(Id)`.
- A distinct value is not a `boolean` condition, ordinal loop variable, or array
  index.
- A distinct type does not satisfy the `Numeric` or `Printable` constraints.
  Generic parameters without a constraint accept distinct values.

Distinct values can be stored in variables, constants, record fields, arrays,
dictionaries, `Option`, `Result`, and other generic containers, and passed to and
returned from routines.

## Units

Public distinct types are exported like other types. Each declaration keeps its
identity across units: two units that each declare `public type Id = distinct integer;`
define two incompatible types. Imported distinct types work with plain and
aliased imports, for example `Ids.UserId(7)` after `uses Demo.Ids as Ids;`.

Distinctness does not validate a value. A `distinct string` for a URL accepts
any string; validate it in a routine that constructs the value.

## See also

- [Type aliases](type-aliases.md)
- [Records](records.md)
- [Generics](generics.md)
