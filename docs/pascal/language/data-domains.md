# Distinct scalars and subranges

These declarations introduce checked, named data domains using ordinary CLR types.
Every value binding still requires an explicit type.

## Distinct types

```pascal
type UserId = distinct long;
type PlayerName = distinct string;

const Id: UserId := new UserId(42L);
const Raw: long := cast<long>(Id);
```

A distinct declaration introduces a nominal type. Neither assignment nor a call
implicitly converts between it and its base type, or between different distinct
types with the same base. Transparent aliases retain the same nominal identity.

The base must resolve through aliases to a non-nullable primitive alias other
than object: a numeric type, bool, char, or string. It cannot be another domain,
an array, an arbitrary class, or a generic parameter. Generic distinct declarations
are not included.

`new UserId(Value)` constructs from the underlying type using ordinary argument
conversion rules. `cast<UserId>(Value)` wraps a base-typed value;
`cast<long>(Id)` unwraps it. The public read-only Value property exposes the same
underlying value. Converting between different domains requires an explicit
intermediate base value. Wrapping/unwrapping does not parse strings or implicitly
change primitive types. Boxing preserves the wrapper's nominal identity.

Same-domain equality compares underlying values. Ordered comparisons are
available for numeric and char bases; string domains compare ordinal contents.
Bool domains have equality only. There is no automatic arithmetic, implicit
printing, or method forwarding. Unwrap before using an API that requires the base.

A value-primitive domain emits an immutable CLR value-type wrapper with private
payload, read-only Value, typed equality, and explicit conversions. Its default
represents the primitive's default; no additional predicate is imposed.

A distinct string emits a sealed immutable reference wrapper. Its value constructor
rejects null with ArgumentNullException; there is no parameterless constructor.
This avoids a default struct containing null while promising a non-null string.
`PlayerName?` is nullable-reference annotation; `UserId?` uses Nullable<UserId>.

## Subranges

```pascal
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

## CLR interoperability

Public domains emit named CLR types, not erased primitive signatures. Other
.NET languages use their constructors, Value, and conversions. Compiler metadata
identifies distinct bases and subrange bounds for cross-assembly FPAS consumers.

Generic constraints inspect the emitted wrapper type; being based on int does
not automatically implement all Int32 interfaces or generic-math traits. Do not
forward arbitrary base members or bypass bounds for generic instantiations.

See the [complete examples](../examples/data-domains.md).
