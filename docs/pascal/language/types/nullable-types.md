# Nullable types

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
  variant equality retains its compiler-defined null-check behavior.
- Use a null branch, `??`, or explicit validation to obtain a non-null value.
  There is no null-forgiving `!` operator or implicit conversion to a variant.

`null` cannot initialize a non-null reference or non-nullable value type.
Unconstrained generic parameters cannot be suffixed with `?`; use an appropriate
reference/value constraint. Repeated nullable suffixes are invalid.
Supported value-type operators also accept nullable operands through the lifted
rules below. This does not permit unchecked use of a nullable reference or an
implicit conversion from T? to T.

## Lifted value operators

Select the existing operator for the contained non-nullable types, retaining
ordinary numeric promotion, constant-expression conversions, and domain rules.
At least one nullable operand lifts that operation. Non-nullable operands may be
mixed with nullable operands. For example, int? + long produces long?, and
short? + byte? produces int?. Real division of integral operands produces
double? when either operand is nullable. Use div for the integral quotient.
Decimal cannot implicitly mix with float or double. Unary numeric promotion
also applies.

| Operator | Result type | Behavior when an operand is absent |
| --- | --- | --- |
| Binary +, -, *, /, div, mod | Nullable form of the ordinary result | null |
| Unary +, - | Nullable form of the ordinary result | null |
| Integral and/or/xor/not, shl/shr | Nullable form of the ordinary result | null |
| Supported enum bitwise operations | Same nullable enum | null |
| = | bool | true only when both operands are absent |
| <> | bool | true only when exactly one operand is absent |
| <, <=, >, >= | bool | false |
| Logical and/or/xor/not on bool? | Unsupported | Handle absence explicitly |

When all operands are present, the ordinary operator determines the result,
including NaN behavior and exceptions. A relational comparison returning false
does not establish the opposite relation: both A < B and A >= B are false if
either operand is absent. Equality is available for bool?; conditions still
require bool. Use Flag = true or Flag ?? false to make that decision explicitly.

Evaluate both binary operands once, left to right, before checking presence.
Only the underlying operation is skipped. Thus an absent dividend divided by
zero produces null, but an exception while calculating the divisor still
propagates. Ordinary checked/unchecked integral, decimal, and floating behavior
applies whenever the operation runs. Suspensions in operands preserve this order.

An untyped null operand takes the known value type of the other operand for an
otherwise valid lifted operation. A null shift count uses int?. An arithmetic
expression with two untyped null operands has no underlying type and is rejected.
Existing direct null tests remain available on references and nullable values.

The same lifting rules apply to imported CLR value-type operators and
constrained generic operators when their parameter and result types are
non-nullable value types. Generic lifting requires a struct constraint
and the appropriate operator interface; struct alone does not supply arithmetic
or equality. Imported default/checked implementations keep their ordinary
dispatch rules. Lifting neither introduces new underlying operators nor erases
distinct domain identity. Reference-type operators retain their existing rules.

Built-in lifted operations are permitted in contracts under the existing
read-only expression rules. Lifting an imported or constrained operator call
does not make that call contract-safe. General lifted comparisons do not prove
that a nullable reference is non-null; ordinary null tests retain their flow rules.

```pascal
program NullableArithmetic;

uses System as Sys, System.Console as Console;

begin
  const Price: decimal? := 12.50M;
  const Missing: decimal? := null;
  const Total: decimal? := Price + 2.50M;
  const Unknown: decimal? := Price + Missing;
  if Total <> 15.00M or Unknown <> null or Missing < Price or Missing >= Price then
    raise new Sys.InvalidOperationException('Incorrect nullable calculation');
  end if;
  Console.WriteLine(Total ?? 0.00M);
  Console.WriteLine(Missing = cast<decimal?>(null));
end;
end program;
```

## Example

```pascal
program NullableText;

uses System.Environment as Environment, System.Console as Console;

begin
  const OptionalName: string? := Environment.GetEnvironmentVariable('FPAS_PLAYER');
  const Name: string := OptionalName ?? 'Explorer';
  Console.WriteLine(Name);

  if OptionalName <> null then
    Console.WriteLine(OptionalName.Length);
  end if;

  const Text: string := 'A😀';
  Console.WriteLine(Text.Length); // 3 UTF-16 code units
  const First: char := Text[0];
  Console.WriteLine(First);       // A
end;
end program;
```

## Flow and imported annotations

Nullability is read from parameter/return/property/field annotations and nullable
contexts. Oblivious reference results remain conservatively nullable.
Unconditional MaybeNull/NotNull return and AllowNull/DisallowNull parameter
contracts are read. Conditional postcondition attributes are not used to prove
flow facts in this implementation.

Null initializes only nullable values/references. A nullable reference cannot
be dereferenced or assigned/passed/returned as non-null without validation.
Stable local/parameter null tests refine if branches and short-circuit and/or
operands. Assignments invalidate/update facts; branch joins retain common facts.
Loops account for assigned locals at their back edge, and exception handlers
use conservative incoming facts. Repeated property reads do not establish
non-nullness. `??` evaluates the fallback only when the left value is absent.

Public source signatures emit proper CLR class/struct/Nullable<T> signatures and
NullableAttribute metadata on nullable reference parameters/results, with the
existing non-null container context. Nested array/generic annotations retain
their positions across FPAS/C# assembly boundaries.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
