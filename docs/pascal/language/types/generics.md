# Generics

Records, enums, functions and procedures declare type parameters with `of (...)`.
Record methods may declare type parameters on the method itself.

Every `of` list is parenthesized, including a single item. This applies to
builtins such as `array of (integer)`, `Result of (integer, string)` and
`dict of (string, integer)`, as well as user-defined applications and generic
declaration headings. Angle-bracket headings and dictionary `to` syntax are
rejected. The existing bare `task` annotation still infers a spawned result;
explicit task result types use `task of (T)`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_params`, `type_param`, `constraint_name`).

## Generic functions and procedures

```pascal
uses Std.Console as Console;

function Identity of (T)(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue of (T)(Value: T);
begin
  Console.WriteLn(Value);
end procedure;
```

Type arguments are inferred from the call-site arguments — no explicit instantiation is needed:

```pascal
var X: integer := Identity(42); // T inferred as integer
var S: string := Identity('hi');

```

## Generic record methods

Record methods declare type parameters in the method header; those parameters are scoped to the method.

```pascal
uses Std.Conv as Conv;

type Box = record
  Value: integer;

  function Map of (R)(Self: Box; F: function(X: integer): R): R;
  begin
    return F(Self.Value);
  end function;
end record;

function ToText(X: integer): string;
begin
  return 'value=' + Conv.IntToStr(X);
end function;

var B: Box := Box(Value := 42);
var S: string := B.Map(ToText);
```

Method-level type parameters may also use constraints:

```pascal
type Accumulator = record
  function Add of (T: Numeric)(Self: Accumulator; Extra: T): T;
  begin
    return Extra;
  end function;
end record;

```

## Implementation

Generics use type erasure. The VM operates on dynamic values, so no monomorphization is needed. Type parameters are checked at compile time and erased at runtime.

## Generic records and enums

```pascal
type Box of (T) = record
  Value: T;
end record;

type Lookup of (T) = enum
  Found(Value: T);
  Missing;
end enum;

var Item: Box of (integer) := Box(Value := 42);
var Present: Lookup of (integer) := Lookup.Found(42);
var Absent: Lookup of (integer) := Lookup.Missing;
```

A type application supplies every argument. Nested applications use the same
form, such as `Box of (array of (Lookup of (integer)))`. Generic arguments are
part of nominal identity: `Box of (integer)` and `Box of (string)` are distinct
types, even if a declaration does not store its parameter.

Constructors infer arguments from supplied fields or positional payloads and the
expected type. A payloadless variant is a value, without parentheses. Missing or
ambiguous arguments require an explicit annotation. Arguments to routines and
variants are positional; named fields belong to record construction.

Constraints apply to data applications and inferred constructors as well as
routine calls. Generic parameter scope belongs to its declaration; resolving a
later header does not inherit another declaration's parameters. Concrete aliases
such as `type IntegerBox = Box of (integer);` preserve nominal identity, defaults
and visibility, including unit reexports. Aliases cannot declare new generic
parameters or receive another argument list.

Recursive records and payload enums must have a finite representable value.
Collections, Option and enum base variants can break a mandatory cycle.
Transparent alias cycles and mandatory cycles without a finite base are errors.

## Constraints

`Equatable`, `Comparable`, `Numeric`, and `Printable` are reserved keywords. They are valid
only after `:` in a generic type-parameter declaration.

Type parameters can be constrained to require specific capabilities from the concrete type. Constraints are written after the parameter name, separated by a colon: `of (T: Constraint)`.

### Built-in constraints

| Constraint | Satisfied by | Description |
|------------|-------------|-------------|
| `Equatable` | Scalars and value data whose components all support equality | Supports `=` and `<>`; resources, tasks and callables are excluded transitively |
| `Comparable` | `integer`, `real`, `boolean`, `string` | Supports comparison operators: `=`, `<>`, `<`, `>`, `<=`, `>=` |
| `Numeric` | `integer`, `real` | Supports arithmetic operators: `+`, `-`, `*`, `/`, `div`, `mod` |
| `Printable` | All types except `function` and `procedure` | Can be converted to a string representation |

### Examples

```pascal
function Max of (T: Comparable)(A: T; B: T): T;
begin
  if A > B then
    return A;
  else
    return B;
  end if;
end function;

function Add of (T: Numeric)(A: T; B: T): T;
begin
  return A + B;
end function;

```

Constraint violations at call sites are compile-time errors:

This includes calls from another generic body. A forwarded type parameter must
declare a constraint that guarantees the required capability. `Numeric` implies
`Comparable`, `Equatable` and `Printable`; `Comparable` implies `Equatable`
and `Printable`, and `Equatable` implies `Printable`. An unconstrained
parameter supplies no constrained guarantee. See
[forwarding generic arguments](../functions/generic-routines.md#forwarding-generic-arguments).

```pascal
var M: integer := Max(3, 7);

```

## See also

- [Record methods](record-methods.md)
- [Generic routines](../functions/generic-routines.md)
