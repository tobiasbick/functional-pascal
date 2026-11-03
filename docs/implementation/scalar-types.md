# Primitive types, aliases, enums, and data domains

The compiler implements the existing [primitive type](../pascal/language/types/README.md),
[literal/arithmetic](../pascal/language/basics/README.md), and
[data domain](../pascal/language/types/data-domains.md) rules directly in CLR metadata
and CIL. The [data-domains example](../../examples/data-domains/README.md) builds
through the SDK; no C# source or FPAS runtime VM is generated.

## Primitive numbers

All specified aliases are available in source declarations, nullable values,
arrays, generic arguments, and CLR signatures: sbyte/byte, short/ushort, int/uint,
long/ulong, nint/nuint, float/double/decimal, plus bool, char, string, and object.
Native integers follow the execution architecture. Numeric literals support
hexadecimal `$FF`, digit separators, integral L/U/UL suffixes, and real F/D/M
suffixes. Unsuffixed integral literals must fit int; a directly negated literal
can express a signed minimum.

Numeric promotions use CLR/C# widening rules. Small integers promote to int for
arithmetic, uint with int promotes to long, and signed integers do not implicitly
combine with ulong. Representable int/long constants can first convert to an
unsigned operand's type, so uint/ulong values can add an unsuffixed positive
literal without changing their type. Decimal cannot implicitly mix with float/double. Default
integral arithmetic and numeric casts check overflow. Unchecked integral operations
wrap; floating conversions follow CLR behavior. Decimal operations/conversions
keep their overflow checks in unchecked expressions. Unsigned division, shifts,
comparisons, and widening preserve unsigned values.

Representable int constants can contextually convert to small integers, unsigned
integers, or native integers; representable long constants can convert to ulong.
Char still requires a character literal or explicit cast. Source optional
parameters preserve exact primitive/enum constants; decimal defaults use the
standard DecimalConstantAttribute. Native integer defaults use ordinary CLR
Int32/UInt32 constant metadata and must fit that architecture-independent width.

## Aliases and enums

Transparent aliases resolve to their target, including nullable/array/generic
targets. They cannot cycle or expose a private type through a public alias.
No CLR type is emitted for an alias. Public aliases retain their exact target
signature and nullable annotations in versioned metadata, so a separate FPAS
assembly can import them; C# sees the target types in ordinary signatures.

Authored enums are actual System.Enum types with a value__ field and literal
members. The base defaults to int and can be any specified fixed-width integral
base. Values auto-increment from zero, can reference other members, and must fit
the base. Numeric casts explicitly wrap/unwrap enum values; bitwise operations
require the same enum. Unnamed values remain legal and matching requires a
fallback. Generic enum declarations are not included.

## Distinct values and subranges

Distinct numeric/bool/char values are sealed immutable CLR structs. Distinct
strings are sealed immutable classes whose constructor rejects null. Each exposes
Value, a constructor, explicit conversions to/from its base, equality, hashing,
and IEquatable<T>. Numeric/char domains also expose ordered operators; string
ordering uses String.CompareOrdinal. Bool domains have equality only. Boxing,
arrays, generic collections, nullable values, and byrefs retain nominal identity.
There is no implicit arithmetic or member forwarding for distinct values.
Nullable value-domain casts lift their conversions. Nullable string payloads and
string-domain receivers require normal null validation before wrapping,
unwrapping, or ordered comparison; equality still permits null references.

Subranges are sealed immutable structs with an integral base, inclusive constant
bounds, base-typed LowerBound/UpperBound fields, and an implicit conversion to
their base. Constructor/cast validation rejects known invalid constants during
compilation and throws ArgumentOutOfRangeException for invalid runtime values,
including inside unchecked. An explicit numeric narrowing before construction
retains its normal conversion behavior.

Their private UInt64 offset encodes the distance from LowerBound using modular
64-bit arithmetic. A zero-initialized wrapper therefore decodes to LowerBound,
including arrays, new T(), and CLR default(T). Signed minimums, full-width signed
ranges, and ranges ending at ulong.MaxValue work without overflow in the encoding.

Integral constant/range patterns support all numeric widths. Subrange patterns
compare the underlying base without constructing a wrapper; coverage can prove
the complete declared interval. Out-of-domain, reversed, and unreachable patterns
are rejected. Ordinary integer and enum selectors still require a fallback.

## Metadata and verification

The [CLR mapping](../specs/clr-mapping.md) defines version 1 of DomainAttribute
and AliasAttribute from Fpas.Core. Importers validate identity/version, storage,
constructors, conversions, bounds, and alias signatures before using the contract.
Fpas.Core contains metadata attributes, not replacement numeric/runtime APIs.
Direct CLI builds must supply it when using these contracts; the SDK handles
references and deployment normally.

Tests execute checked/unchecked arithmetic, decimal/native/unsigned conversions,
nullable/array values, enum constants, equality/hashing/ordering, default domain
values, and boundary failures. A mixed C#/FPAS build references the model library
from a separate FPAS consumer and C# host, with nullable warnings as errors.
Metadata tests reject corrupted contracts and verify deterministic emission.
XML documentation emits real type/enum-field IDs and uses alias target types in
method IDs; an alias itself has no CLR type documentation ID.

[Global fields and initialization](global-values.md) are implemented.
[Named nested routines](nested-routines.md) are implemented.
[Typed/repeat loops](loops.md) are implemented.
[Source attributes](attributes.md) are implemented.
Authored interfaces, fully general nested generic type paths, and LSP/query tooling remain
outside this milestone. Numeric nullable lifted arithmetic is also not included.
