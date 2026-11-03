# Unions and case matching

This milestone adds nongeneric unions and case matching to
[records](records.md), following the approved [union types](../pascal/language/types/unions.md)
and [control-flow rules](../pascal/language/pattern-matching/syntax.md).
The [game-actions example](../../examples/game-actions/README.md) compiles and runs
through its project and uses records as union payloads.

```pascal
unit Actions;

public type Outcome = union
  Moved(X: int; Y: int);
  Blocked(Reason: string);
  Stayed;
end union;

public function Describe(Value: Outcome): string;
begin
  return case Value of
    when Outcome.Moved(const X: int, const Y: int):
      X.ToString() + ',' + Y.ToString();
    when Outcome.Blocked(const Reason: string): Reason;
    when Outcome.Stayed: 'stayed';
  end case;
end function;

end unit;
```

## Construction, equality, and visibility

A union requires one or more named cases. Payload declarations use explicit
types, separated by semicolons. Payload construction uses comma-separated
arguments: Outcome.Moved(1, 2) or Outcome.Moved(Y := 2, X := 1). Positional
arguments may precede named arguments. Supplied arguments run once in written
order; every payload is required. A fieldless case is a value property,
Outcome.Stayed, with no parentheses. Constructing the abstract union with new
is an error. Union payloads cannot be changed with record with updates.

Unions may be private or public in units; programs have private types. Private
union types cannot appear in public signatures. Payload properties are readonly.
Records, arrays, ordinary CLR types, delegates, nullable values, and recursive
unions can be payloads within the existing type subset. All finite values must
be constructed through the declared cases; null requires a nullable annotation.

Equality compares the actual case and its fields using EqualityComparer<T>.Default.
It implements IEquatable<T>, typed/object Equals, GetHashCode, and equality
operators, including null behavior. Different cases do not compare equal. Mutable
objects held by payloads retain their normal CLR semantics; no deep copy occurs.
Cases and payload names cannot collide with generated members or nested case
type names.

## Statements, expressions, and patterns

Both case statements and expressions evaluate their selector once and test arms
in source order. A statement arm contains statements. An expression arm contains
one expression followed by a semicolon, as above. Results need a common type.
There is no fallthrough. `else` introduces a fallback without a colon.

Implemented patterns:

- Literals and CLR metadata constants of supported primitive/enum types, including
  signed numeric constants. Numeric constant patterns use numeric equality;
  Floating NaN is rejected as a constant pattern because it never equals itself.
- Inclusive integral/char constant ranges, such as 0..9 or #65..#90,
  including all integer widths and declared subranges.
- `null`, `_`, and `const Name: T` whole-value bindings.
- Union cases with positional or named payload patterns, including nested cases.
- CLR type tests and non-null bindings: `is Sys.Exception as Failure`.
- An optional `if Condition` guard after a complete pattern or alternative list.
- Comma-separated alternatives with identical binding names and types.

Every payload must be matched, using _ to ignore it. Named payloads may be
reordered, but cannot mix with positional payloads. An explicit const annotation
must match the selected static type exactly; it does not perform a conversion or
type filter. For nullable whole-value bindings, retain the nullable annotation.
Bindings exist only in their arm and guard and are immutable; lambdas may capture
them normally. A shared guard runs once after the first matching alternative.
A false guard continues at the next arm.

Constant matching does not call arbitrary user-defined equality operators or
constructors. Matching a boxed primitive also checks its actual CLR type. Type
patterns use CLR isinst semantics, including supported boxing and interface/array
assignability. `Value is T` is also available outside case patterns as a Boolean
test, without introducing a binding; a true test proves a stable reference non-null.

Case statements support return, break, continue, exceptions, and using/finally
cleanup through the normal control-flow lowering. Nullable facts are merged
across continuing arms, and callback/guard side effects invalidate captured
mutable facts. A case whose arms all return satisfies the normal return check.

## Coverage and unreachable arms

Both forms require exhaustive matching. Unions require every case, descending
into nested payload patterns; Boolean selectors require true and false. Nullable
selectors additionally require null coverage. Other primitive/enum selectors and
open CLR hierarchies require a fallback. Covering an int's entire machine range
does not make it a declared subrange domain. Guards never establish coverage.

Duplicate or already covered patterns are errors, including unreachable fallbacks.
Analysis preserves relationships between payload columns rather than checking
each field independently. For example, all four combinations of a pair of bool
payloads cover that case. Missing one combination remains an error. The analysis
has a bounded work limit and asks for simpler nested patterns if exceeded.

The selector's declared local/parameter/capture type determines coverage even
when flow already knows that its current value is non-null. An impossible match
at runtime, such as foreign code violating annotations, throws an ordinary
InvalidOperationException. It does not fabricate a payload or use an unchecked cast.

Diagnostics: FP3023 covers union declarations/construction forms, FP3024 covers
invalid patterns/bindings/constants, FP3025 covers coverage/unreachable patterns.
Named construction reuses FP3022; malformed imported metadata produces FP4001.

## CLR and assembly boundaries

Outcome becomes a nested abstract CLR class in its unit container. It has a
private protected constructor, an abstract readonly int Tag property, and nested
sealed MovedCase, BlockedCase, and StayedCase classes. Payload cases have public
static factories; Stayed is a public static property. Case constructors are
internal, and payload properties retain source names. C# can call the factories
and match `value is Actions.Outcome.MovedCase moved`, then read moved.X/moved.Y.
External C# subclasses cannot call the union constructor. Tag values follow
declaration order and are not a persistent storage schema.

A second FPAS project can import the public union, construct cases, and perform
exhaustive matching using the versioned Fpas.Core union/case attributes. Metadata
validation checks the owner, class/constructor shape, readonly payloads, exact
factory/getter signatures, payload labels, nullable annotations, and contiguous
tags. Ordinary CLR classes and C# records do not become closed language unions.
The SDK supplies and copies the matching Core assembly as described for
[records](records.md#clr-and-assembly-boundaries). The exact metadata ABI is in the
[CLR mapping](../specs/clr-mapping.md#records-and-unions).

## Verification and remaining work

Tests execute source and imported unions, named arguments, nested/guarded patterns,
alternative bindings, closures, nulls, ranges, type tests, and protected control
transfers. A truth-table oracle checks all 512 selections of pair-of-bool patterns
independently of the pattern-matrix implementation. Metadata corruption tests
reject incompatible versions, duplicate tags, and an externally accessible base
constructor. A mixed C#/FPAS build verifies factories, concrete cases, equality,
reflection, and rejection of external subclassing. The game-action project is
built and executed as an integration test.

[Generic source functions and record methods](generic-routines.md) now support
explicit type arguments and constraints. [Generic unions and records](generic-types.md)
now support recursive typed data and cross-assembly matching.
[Constructor inference](type-argument-inference.md) also accepts uninstantiated
union owners, with payload and expected-type evidence.
[Primitive numbers, authored enums, and scalar domains](scalar-types.md) are implemented;
subrange coverage can prove a complete declared interval.
LSP/query tools remain outside this milestone. There are
no compiler-provided Option/Result types or automatic error propagation. Domain
unions model ordinary outcomes; exceptional failures retain CLR exceptions.
[XML documentation](xml-documentation.md) is implemented.
[Source overloads, async, and portable PDBs](overloads-interfaces-debugging-and-async.md)
are implemented.
