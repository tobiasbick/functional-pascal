# Types and units

Read this when declaring data, generics, domain types or a multi-unit program.

## Records and enums

Records are nominal value types. Construct them through their type with named
fields: `Point(X := 1, Y := 2)`. Omit only fields with defaults; `Point()` uses
all defaults or constructs an empty record. Supplied values execute in written
order, then omitted defaults in declaration order. `Base with X := 3; end with`
returns an updated copy. A `var` record also permits field assignment.

Methods are declared inside the record. An instance method declares read-only
`Self: RecordType` first and omits it at the call site. A static routine has no
Self and is called through the type. A bound method captures the receiver value
once. Dot calls find declared members or fixed native operations; they never
search free routines by their first parameter. Postfix fields, indexes and calls
can follow expression results.

Enums list semicolon-terminated variants. Fieldless enums may assign unique
integer backing values. Data enums declare payload fields separated by semicolons
and cannot have backing integers. Qualify variants (`Shape.Dot`, `Shape.Circle(3)`),
including in comparisons and patterns. Constructors accept positional or named
arguments. Fieldless variants have no parentheses. See [patterns](patterns-errors.md).

Equality is structural for records, enums, Option and Result when all contained
values support equality. Whole arrays, dictionaries, callables, tasks and channels
do not. Equality-capable aggregate types do not necessarily support ordering.

## Generics and scalar domain types

Type declarations and applications use `of`: `type Box of T = record ...`,
`Box of integer`, or `Pair of (integer, string)`. Routines declare angle-bracket
parameters, such as `function Identity<T>(Value: T): T;`. Calls infer arguments
from supplied values and fill missing ones from the expected result type.
Explicit angle-bracket call arguments are unavailable. Constructors use the plain
type name, as in `Box(Value := 3)`, with inference or an expected type.

Constraints are `Numeric`, `Comparable`, `Printable`, and `Error`, written
`T: Comparable`. Numeric permits integer/real arithmetic and implies Comparable
and Printable; generic `div`/`mod` are unavailable because T may be real.
Comparable supports scalar ordering, including fieldless enums, distinct scalars
and subranges. Printable permits output. Error accepts native error families,
implies Printable and exposes `.Message()`. Unconstrained T permits storage and
passing. Defaults must be valid for every instantiation.

`type Count = integer;` is a transparent alias.
`type UserId = distinct integer;` creates a separate scalar identity; use
`UserId(Value)` to wrap and `integer(Id)` to unwrap. Distinct wraps primitive
integer/real/boolean/string types, supports same-type comparisons, and satisfies
Comparable. It does not implicitly inherit arithmetic, printing or native methods.

`type Percent = 0..100;` declares inclusive constant integer bounds.
`Percent(Value)` checks narrowing; widening to integer is implicit. Arithmetic
returns ordinary numbers, so explicitly narrow to validate the result.
`Value in Percent` tests bounds. Equal-bound subranges are compatible and satisfy
Comparable and Printable, but not Numeric. Convert real through `Std.Math`
rounding operations rather than `integer(...)`.

```pascal id=data-and-generics run
program AgentTypes;

uses Std.Test as Test;

type Box of T = record
  Value: T;
end record;

type UserId = distinct integer;

type Percent = 0..100;

type Shape = enum
  Dot;
  Circle(Radius: integer);
end enum;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

begin
  const Wrapped: Box of integer := Box(Value := Identity(4));
  const Updated := Wrapped with Value := 5; end with;

  Test.AssertEquals(4, Wrapped.Value);
  Test.AssertEquals(5, Updated.Value);
  Test.AssertEquals(7, integer(UserId(7)));
  const Half: Percent := Percent(50);
  Test.AssertTrue(Half in Percent);
  Test.AssertTrue(Shape.Circle(3) is Shape.Circle(_));
end.
```

Types are visible throughout their program/unit. Recursive types need finite
values: recursive enums need a base variant, and recursive records can use an
Option field. Recursive generic references retain the same type arguments.

## Units, imports and visibility

A unit starts with `unit Qualified.Name;`, then optional imports and declarations,
and ends with `end unit;`. It has no program main block or separate
interface/implementation sections. The `Std` namespace root belongs to the
standard library. Import with `uses App.Values as Values;`; names are accessible
only through that source-local alias. Import each unit once, case-insensitively.
Aliases cannot be shadowed. Project dependencies make sources discoverable;
they do not replace imports.

Each unit declaration is private unless marked `public`. Record members require
their own public markers. A record with private fields can only be constructed
inside its unit; export a factory. Consult the record handbook before exporting
computed field defaults.

The following companion files belong in one program project:

```pascal id=unit project=agent-units file=values.fpas
unit Agent.Values;

public type Number = record
  public Value: integer := 0;
end record;

public function Create(Value: integer): Number;
begin
  return Number(Value := Value);
end function;
end unit;
```

```pascal id=consumer project=agent-units file=consumer_test.fpas run
program AgentUnits;

uses Agent.Values as Values, Std.Test as Test;

begin
  const Number := Values.Create(7);
  Test.AssertEquals(7, Number.Value);
end.
```

Further rules: [records](../language/types/records.md),
[generics](../language/types/generics.md), [units](../program-structure/units.md),
[projects](../program-structure/projects.md), and
[workspaces](../program-structure/workspaces.md). Source manifests are authoritative;
`.fpascu` and `.fpascp` files are derived build outputs.
