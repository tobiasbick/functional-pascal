# Generics

Records and enums declare type parameters after `of`; routines and record methods declare
their own parameters in angle brackets (`<T>`).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf)
(`data_type_params`, `data_type_arguments`, `type_params`, `type_param`, `constraint_name`).

## Built-in type applications

Type applications use `of`. Single-argument types have no argument parentheses:
`array of integer`, `Option of string`, `channel of integer`, and `task of integer`.
`Result` requires exactly two arguments in parentheses:
`Result of (integer, string)` or `Result of (Option of User, string)`.
Nested applications keep their own delimiters, for example
`Result of (Result of (integer, string), Option of boolean)`.
The dictionary form remains `dict of string to integer`.

Unparenthesized `Result` arguments and angle-bracket type applications are
syntax errors. Angle brackets declare routine type parameters; they do not
apply types. User-defined generic records and enums use the same `of` form.

## Generic records

```pascal
type Box of T = record
  Value: T;

  function Get(Self: Box of T): T;
  begin
    return Self.Value;
  end function;
end record;

type Pair of (K: Comparable, V) = record
  Key: K;
  Value: V;
end record;

const Number: Box of integer := Box(Value := 42);
const Entry: Pair of (integer, string) := Pair(Key := 1, Value := 'one');
```

A single parameter or argument has no parentheses. Multiple parameters or
arguments require parentheses. Arguments may themselves be containers, callable
types, or generic records and enums, such as `Box of Lookup of array of integer`.
Arity and constraints are checked at each application. A type parameter used as
an argument must guarantee the required constraint; `Numeric` also guarantees
`Comparable` and `Printable`.

Each instantiation retains the record's nominal identity and its arguments.
`Box of integer` and `Box of string` are incompatible. Aliases retain the same
arguments, defaults, and member visibility. Instance methods declare the
instantiated receiver explicitly, as `Self: Box of T` above. Record parameters
are available in field types, defaults, and method signatures and bodies.
Method inference preserves the receiver's arguments. A method's own parameters
have their own scope, even when a caller uses the same parameter names.

### Constructor inference

Construct with the ordinary name and named fields: `Pair(Key := 1, Value := 'one')`.
Explicit applications in calls, such as `Pair of (integer, string)(...)`, are
errors; place the arguments in a type annotation or alias.

All supplied field values contribute to inference together. Their written order
does not affect the inferred arguments. An expected type from an annotation,
assignment, return, enclosing field, or routine parameter fills undetermined
parameters. It must agree with arguments determined by the supplied values.
For a generic routine call, arguments that determine the routine's parameters
also supply that expected type: `Accept(Optional(), 1)` can determine the missing
record argument from a parameter shared with the second argument.
Generic routine values in callable fields are instantiated from the field's
expected signature after all supplied fields have contributed.
Conflicting values are errors; inference does not promote numeric types or
search for a common type. Parameters left undetermined require a type annotation.

```pascal
type Optional of T = record
  Value: Option of T := None;
end record;

const EmptyValue: Optional of string := Optional(); // Annotation determines T.
const Present: Optional of integer := Optional(Value := Some(3));
```

Defaults are checked in the declaration's scope under its parameter constraints
and must work for every allowed argument. `Value: T := 0` is invalid for an
unconstrained `T`; `Value: Option of T := None` is valid. Defaults never infer an
argument, so `Optional()` without an expected type is underdetermined.
[Record construction](records.md#creating-a-record) preserves visibility and
evaluates supplied fields once in written order, followed by omitted defaults
in declaration order.

### Recursive records

```pascal
type Node of T = record
  Value: T;
  Next: Option of Node of T := None;
end record;
```

Every generic reference in a recursive type cycle must forward the declaring
parameters unchanged and in the same order. Mutually recursive declarations may
use different parameter names. Reordered, replaced, or wrapped arguments are
errors. These records must also satisfy the
[finite-construction rules](declaration-order.md): unchanged arguments alone do
not make a mandatory stored-value cycle valid.

See the runnable [generic records example](../../../../examples/pascal/types/generic_records.fpas).

## Generic enums

```pascal
type Lookup of T = enum
  Found(Value: T);
  Missing;
end enum;

type Choice of (L, R) = enum
  Left(Value: L);
  Right(Value: R);
  Neither;
end enum;

const Number: Lookup of integer := Lookup.Found(42);
const Text: Lookup of string := Lookup.Found(Value := 'hello');
const Missing: Lookup of string := Lookup.Missing;
const Partial: Choice of (integer, string) := Choice.Left(1);
```

Enum parameters have the same declaration, application, constraint, and nominal
compatibility rules as record parameters. `Lookup of integer` and `Lookup of
string` are incompatible. Aliases and imported types preserve their arguments
and the identity of the declaring enum.

Construct variants using ordinary names, with all payload fields positional or
all named. Explicit applications such as `Lookup of string.Found('hello')` are
errors; write an annotation or use an alias. All supplied payloads contribute to
inference together, regardless of named-field order. The expected enum type fills
missing arguments and must agree with the supplied values. A payloadless variant
such as `Lookup.Missing`, or `Choice.Left(1)` with undetermined `R`, requires that
context. An annotation, assignment, return, enclosing field, or routine parameter
can supply it. Arguments to a generic routine can supply shared type evidence.
Conflicting evidence is rejected without finding a common type or promoting
numeric types. Payload values are evaluated once in written order.

### Recursive enums and matching

```pascal
type List of T = enum
  Empty;
  Cons(Head: T; Tail: List of T);
end enum;

function Length<T>(Values: List of T): integer;
begin
  return case Values of
    when List.Empty: 0;
    when List.Cons(_, const Tail): 1 + Length(Tail);
  end case;
end function;

const Numbers: List of integer := List.Cons(1, List.Cons(2, List.Empty));
```

Every reference in a recursive cycle forwards the declaring parameters unchanged
and in the same order, including cycles through records or mutually recursive
enums with different parameter names. Recursive applications such as `List of
array of T`, concrete replacements, and reordered parameters are rejected even
when another variant could terminate the cycle. The ordinary
[finite-construction rules](declaration-order.md) then apply: an enum must have a
terminating alternative. `List.Empty` provides one; a sole mandatory recursive
payload does not.

[Patterns](../pattern-matching/enum-patterns.md) use ordinary variant names and
the matched value's arguments. Bound payloads have the substituted types, and
patterns may nest through enums, records bound as values, Result, and Option.
An alias fixed to different arguments cannot match the value. Cases retain
[closed-enum exhaustiveness](../pattern-matching/exhaustiveness.md), including
missing nested alternatives and the rejection of `else`.

See the runnable [generic enums example](../../../../examples/pascal/types/generic_enums.fpas).

## Generic functions and procedures

```pascal
function Identity<T>(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue<T>(Value: T);
begin
  WriteLn(Value);
end procedure;
```

Type arguments are inferred from the call-site arguments — no explicit instantiation is needed:

```pascal
const X: integer := Identity(42); // T inferred as integer
const S: string := Identity('hi'); // T inferred as string
```

## Generic record methods

Record methods declare type parameters in the method header; those parameters are scoped to the method.

```pascal
type Box = record
  Value: integer;

  function Map<R>(Self: Box; F: function(X: integer): R): R;
  begin
    return F(Self.Value);
  end function;
end record;

function ToText(X: integer): string;
begin
  return 'value=' + IntToStr(X);
end function;

const B: Box := Box(
  Value := 42
);
const S: string := B.Map(ToText); // R inferred as string
```

Method-level type parameters may also use constraints:

```pascal
type Accumulator = record
  function Add<T: Numeric>(Self: Accumulator; Extra: T): T;
  begin
    return Extra;
  end function;
end record;
```

## Implementation

Generics use type erasure. The VM operates on dynamic values, so no monomorphization is needed. Type parameters are checked at compile time and erased at runtime.

## Constraints

`Comparable`, `Numeric`, and `Printable` are reserved keywords. They are valid
only after `:` in a generic type-parameter declaration.

Type parameters can be constrained to require specific capabilities from the concrete type. Constraints are written after the parameter name, separated by a colon: `<T: Constraint>`.

### Built-in constraints

| Constraint | Satisfied by | Description |
|------------|-------------|-------------|
| `Comparable` | `integer`, `real`, `boolean`, `string`, [distinct types](distinct-types.md) | Supports comparison operators: `=`, `<>`, `<`, `>`, `<=`, `>=` |
| `Numeric` | `integer`, `real` | Supports arithmetic operators: `+`, `-`, `*`, `/`, `div`, `mod` |
| `Printable` | All types except `function`, `procedure`, and distinct types | Can be converted to a string representation |

### Examples

```pascal
function Max<T: Comparable>(A: T; B: T): T;
begin
  if A > B then
    return A;
  else
    return B;
  end if;
end function;

function Add<T: Numeric>(A: T; B: T): T;
begin
  return A + B;
end function;
```

Constraint violations at call sites are compile-time errors:

```pascal
const M: integer := Max(3, 7); // Valid — integer is Comparable
// var Bad := Max([1], [2]);   ← compile error: array is not Comparable
```

## See also

- [Record methods](record-methods.md)
- [Generic routines](../functions/generic-routines.md)
