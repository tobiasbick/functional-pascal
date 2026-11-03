# Types

## CLR primitives

These aliases denote the actual CLR types, not wrappers.

| Alias | CLR type |
| --- | --- |
| `bool` | `System.Boolean` |
| `byte`, `sbyte` | `System.Byte`, `System.SByte` |
| `short`, `ushort` | `System.Int16`, `System.UInt16` |
| `int`, `uint` | `System.Int32`, `System.UInt32` |
| `long`, `ulong` | `System.Int64`, `System.UInt64` |
| `nint`, `nuint` | `System.IntPtr`, `System.UIntPtr` |
| `float`, `double`, `decimal` | `System.Single`, `System.Double`, `System.Decimal` |
| `char` | `System.Char`, one UTF-16 code unit |
| `string`, `object` | `System.String`, `System.Object` |

Procedures return `System.Void`, which is not a general source value type.
Native-sized integers follow the target architecture. `string` is immutable;
its `Length` and indexes use UTF-16 code units. `Text[Index]` returns `char`.
Use imported .NET APIs for Unicode scalars, graphemes, encoding, and formatting.

There are no implicit boxing-based collection conversions or implicit narrowing
numeric conversions. Ordinary CLR boxing to `object` or an implemented interface
is supported. Explicit unboxing uses `cast<T>` and requires the actual boxed type.

## Names, generics, and aliases

```pascal
type UserName = string;
type Names = Collections.List<string>;
type Matrix = double[,];
```

`type Name = T;` is a transparent source alias and does not emit a new CLR type.
It may not be recursive. Type arguments uniformly use `<...>`:
`List<string>`, `Task<string>`, and `Dictionary<string, int>`.
Nested generic closers such as `List<List<int>>` are valid.
Type names and generic arity are resolved exactly. Source declarations cannot
differ solely by capitalization accidentally: capitalization denotes distinct
names and is never folded by import lookup.

Use `type UserId = distinct long;` for a nominal scalar identity and
`type Percent = int range 0..100;` for a checked integral subrange. Both have
defined CLR representations and conversions; see [data domains](data-domains.md).

## Nullable types

- `T?` for a non-nullable value type means `System.Nullable<T>`.
- `T?` for a reference type is a nullable reference annotation; it does not
  construct `Nullable<T>` or a wrapper object.
- `T` reference values are statically non-null. Definite assignment and nullable
  flow analysis diagnose possible null dereferences and invalid assignments.
- Nullable metadata is imported and emitted. Unannotated foreign reference
  results are conservatively nullable; external annotations are contracts, not
  a guarantee that external code cannot return null.
- A null test on a stable local/parameter refines it for the applicable branch
  and the corresponding short-circuit Boolean operand. A mutable local captured
  by a closure, or invalidated by assignment/byref use, is not stable. Reading a
  mutable property twice does not establish non-nullness.
- A foreign overloaded equality operator can define its own null behavior and
  does not establish non-nullness. A runtime `Value is T` check can validate a
  nullable reference independently of such an operator. Generated FPAS record/
  union equality retains its compiler-defined null-check behavior.
- Use a null branch, `??`, or explicit validation to obtain a non-null value.
  There is no null-forgiving `!` operator or implicit conversion to a union.

`null` cannot initialize a non-null reference or non-nullable value type.
Unconstrained generic parameters cannot be suffixed with `?`; use an appropriate
reference/value constraint. Repeated nullable suffixes are invalid.

## Records

```pascal
public type Point = record
  X: double;
  Y: double;

  function Length(): double;
  begin
    return Math.Sqrt(self.X * self.X + self.Y * self.Y);
  end function;
end record;
```

A language record is a nominal, sealed CLR **reference type** with read-only
properties. It is not a CLR struct and has no implicit value-copy-on-assignment
operation. Assignment shares the reference; immutability makes sharing safe for
the record's own fields. Referenced objects inside fields may still be mutable.

Fields and methods are public by default within the record's accessibility.
Explicit `public` is allowed; `private` restricts a member to code lexically
inside that record, including static methods. Other code, even in the same unit,
must use its public API. All fields remain read-only regardless of visibility.

Construct with `new Point(X := 3.0, Y := 4.0)` or `new Point(3.0, 4.0)`.
Declaration order defines positional constructor order. A field without a default
is required. A field may declare `Name: T := Expression;`; all field types are
explicit. Supplied arguments evaluate in written order, followed by omitted
defaults in declaration order, once per construction. Defaults execute only for
omitted fields. Use named arguments to omit a non-trailing defaulted field.

Defaults use the record's declaration scope and generic parameters, not the
caller's locals. They cannot read self or other instance fields before construction.
They may call routines or read globals under normal initialization rules, and may
throw. A default must type-check for every valid generic instantiation. Defaults
are not implicit CLR parameter constants.

If any field is private, the generated all-fields constructor is private: only
code inside the record can construct it, even if that field has a default.
A public static factory may expose selected construction choices. There is no
custom constructor-body syntax; factories are ordinary methods with optional contracts.
Field defaults do not synthesize additional CLR constructor overloads: satisfying
a CLR `new()` constraint still requires an actual accessible parameterless
constructor, not just an FPAS call with all field arguments omitted.

`P with X := 8.0; end with` constructs a shallow updated record, evaluating `P`
once and updates in written order. It does not clone referenced collections.
Duplicate/unknown/inaccessible update fields are errors. Unmentioned fields,
including private fields, are retained; defaults do not rerun. Generated copy
helpers permit external updates of accessible fields without exposing private
payloads. The operand must be a language record, not an arbitrary imported CLR object.

Instance methods have an implicit read-only `self` binding. `static function` or
`static procedure` declares a type-level method with no self, called through the
record type. Both kinds may be private/public, generic, and have contracts;
static functions may be async under the ordinary rules. Static methods do not
become interface instance implementations. Free functions remain the primary
place for application behavior. There are no user-authored class hierarchies.

For default metadata and constructor/copy emission see
[CLR mapping](../../specs/clr-mapping.md). Complete examples are in
[record members](../examples/record-members.md).

Records generate typed equality, `Equals`, and `GetHashCode`. Corresponding fields
use `EqualityComparer<T>.Default`; array fields therefore compare by reference,
not element-by-element. Equality is not a recursive deep comparison of arbitrary
.NET object graphs. Null equals null; a non-null record never equals null.
Source members may not collide with the constructor, generated equality members,
or emitted property/accessor identities; diagnose these collisions explicitly.

## Discriminated unions

```pascal
public type Shape = union
  Dot;
  Circle(Radius: double);
  Rectangle(Width: double; Height: double);
end union;
```

Construct values with `Shape.Dot`, `Shape.Circle(2.0)`, or
`Shape.Rectangle(Width := 3.0, Height := 4.0)`. A fieldless case has no parentheses.
Payload fields are immutable. Case names and payload labels are case-sensitive.

A union is a closed nominal reference type with compiler-generated case
representations. It is **not** `System.Enum`. Generic unions use ordinary type
parameters. `Tree<T>` may recursively refer to `Tree<T>` because CLR references
break the storage cycle. All paths to constructing finite values must still
follow the declared constructors; null is not a union case.

Generic unions are declared by applications or imported from libraries like
other types. For example:

```pascal
type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;
```

Constructors select the uninstantiated union owner, for example `Tree.Leaf(1)`.
Type arguments are inferred from payloads and expected type. A fieldless generic
case requires expected type, such as `const Empty: Tree<int> := Tree.Empty;`.
Constructor inference must resolve exactly one union type; arbitrary class static
methods do not gain this shorthand. Value declarations still require explicit
types even when a constructor supplies enough information to infer its arguments.

Union equality compares the case and its fields using the record field rules.
[Control flow](control-flow.md) specifies exhaustive matching.

## Absence, failures, and domain outcomes

Use `T?` for nullable values and CLR exceptions for failures. A domain operation
can return its own union when several normal outcomes have different payloads;
see the [gameplay example](../examples.md#6-gameplay-outcomes-as-an-application-union).
Such a union is ordinary data: it neither unwinds the stack nor causes an implicit
return when one case is encountered. Inspect it with case matching.

Option and Result are not predefined types, implicit imports, or required
Fpas.Core APIs. A future library or application may define them as ordinary
unions, with no compiler-specific failure behavior. The language has no postfix
failure-propagation `?`; the `?` type suffix denotes CLR nullability.

## CLR enums

```pascal
type Direction = enum : byte
  North = 0;
  East = 1;
  South = 2;
  West = 3;
end enum;
```

An enum is an actual CLR enum with underlying type `byte`, `sbyte`, `short`,
`ushort`, `int`, `uint`, `long`, or `ulong`, default `int`.
Explicit values must fit that type; omitted values increment the previous value,
starting at zero. Duplicate values are allowed for CLR aliases. Payloads are not
allowed. Use `Direction.North` and explicit casts to/from the underlying type.

CLR enum values may contain unnamed underlying values. Exhaustive union rules do
not apply to enums. Same-type bitwise operations are supported; a `[FlagsAttribute]`
annotation documents intended flags use but does not restrict runtime values.

## Arrays and collections

`T[]` is a one-dimensional zero-based CLR vector; `T[,]` is a rectangular
two-dimensional array; `T[][]` is jagged. Allocation uses `new int[Count]` or
`new double[Rows, Columns]`. Indexes are `Values[I]` and `Grid[X, Y]`.
All dimensions are evaluated once and must be valid `int` lengths.

New arrays use CLR default element initialization. For reference elements, an
allocation with potentially nonzero length requires a nullable element annotation;
use `new string?[Count]`. A non-null element vector can instead be constructed
with a fully initialized literal. The same rule applies to default-null elements
of rectangular arrays. Imported APIs retain their declared annotations and may
violate them like other external contracts. Jagged arrays can be constructed
from vector literals or through an element type alias; no special jagged `new`
suffix syntax is included in this baseline.

`[1, 2, 3]` is an array literal. Empty `[]` requires an expected vector type.
Array elements are mutable. Assigning an array shares it, including through
`const` bindings. Array bounds errors are CLR exceptions. Array literals are not
automatically lists or dictionaries.

Use actual .NET collections:

```pascal
const Names: Collections.List<string> := new Collections.List<string>();
Names.Add('Mara');
const Scores: Collections.Dictionary<string, int> := new Collections.Dictionary<string, int>();
Scores['Mara'] := 10;
```

Members retain CLR meanings: `Length`, `Count`, `Add`, `TryGetValue`, `GetLength`,
and so on. There are no compiler-supplied `PushInPlace`, `dict`, or collection
snapshot semantics. Ordering and mutation during enumeration follow each API.
