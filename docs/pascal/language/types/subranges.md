# Subranges

```pascal fragment
type Percent = int range 0..100;
type PortNumber = ushort range 1..65535;

const Level: Percent := new Percent(50);
const Number: int := cast<int>(Level);
```

A subrange has an explicit integral base and inclusive compile-time bounds.
Its base resolves to byte, sbyte, short, ushort, int, uint, long, or ulong.
Native-sized integers, enums, and generic bounds are not included. Bounds must
fit that base and Lower <= Upper. Literal bounds use the base as expected type;
named bounds must be compile-time constants.

Each declaration introduces a nominal CLR type. Equal bounds do not make two
separately declared types identical; aliases of one declaration remain compatible.
Crossing domain identities requires explicit unwrapping and checked construction.

`new Percent(Value)` and `cast<Percent>(Value)` validate a base-typed value.
A statically known violation is a compile-time error; a runtime value outside
the interval throws ArgumentOutOfRangeException. A narrowing numeric conversion
performed explicitly before wrapping throws OverflowException first if it cannot
produce a base value. Domain construction does not implicitly narrow its input.
`unchecked(...)` never suppresses domain-bound validation.

A subrange implicitly widens to its base, followed by ordinary numeric widening
where applicable. Arithmetic produces the promoted base type, not the subrange.
Explicitly construct/cast to validate a new domain value. No implicit narrowing
into a subrange occurs. Same-domain comparison uses underlying numeric values.

Value returns the base value. Generated LowerBound and UpperBound fields are
base-typed compile-time constants. Use ordinary comparisons to validate input;
this does not introduce expression-level membership syntax. An array element,
global, or ref parameter retains the domain type: it cannot be passed as
`ref int` to mutate a payload unchecked.

### CLR default-value invariant

A subrange is an immutable CLR value type. Its zero-initialized/default
representation must represent **LowerBound**, even when zero is outside the
interval. This includes default array elements and generic .NET code.

Store a private unsigned offset from the lower bound. Zero offset denotes
LowerBound. Constructor checks precede encoding; Value decodes the offset.
UInt64 can represent the width of any supported interval, including the full
Int64 or UInt64 interval. Internal encoding/decoding must handle signed extremes
without overflowing for a valid domain value.

An element of `new PortNumber[2]`, or a parameterless `new PortNumber()`, therefore
has Value 1. There is no invalid default sentinel. Private readonly storage and
checked constructors/conversions preserve this invariant through supported managed
operations. Unsafe/reflection
corruption is outside the language's guarantees.

## Example

```pascal
program Ranges;

uses System as Sys, System.Console as Console;

type Percent = int range 0..100;
type PortNumber = ushort range 1..65535;

begin
  const Level: Percent := new Percent(50);
  const Increased: int := Level + 10;
  const Next: Percent := new Percent(Increased);
  Console.WriteLine(cast<int>(Next)); // 60

  const Ports: PortNumber[] := new PortNumber[2];
  Console.WriteLine(Ports[0].Value); // 1: the valid lower bound, not zero

  var Input: int := int.Parse('101');
  try
    const Invalid: Percent := new Percent(Input);
    Console.WriteLine(Invalid.Value);
  catch Failure: Sys.ArgumentOutOfRangeException do
    Console.WriteLine(Failure.Message);
  end try;
end;
end program;
```

The Parse call supplies a runtime value. A literal `new Percent(101)` is instead
a compile-time bound violation. Neither unchecked arithmetic nor byref access to
the underlying primitive can bypass range validation.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
