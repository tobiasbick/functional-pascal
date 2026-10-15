# Subrange types

A subrange restricts an `integer` to inclusive compile-time bounds:

```pascal id=example-1 fragment context=declarations
type Percent = 0..100;

type Signed = -10..10;

type Single = 1..1;
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`subrange_type`).

Both bounds must be [compile-time integer constants](../basics/constants.md#compile-time-constants).
Static expressions, preceding constants, and imported static constants are
accepted. Calls and computed constants are rejected. The lower bound must not
exceed the upper bound. There is no explicit base-type annotation; real,
Boolean, string, and enum bounds are invalid.

## Checked conversions

Call the type with exactly one positional integer or subrange value:

```pascal id=example-2 fragment context=subrange-values
const Low: Percent := Percent(0);
const High: Percent := Percent(100);
```

A known value outside the bounds is a compile-time error. A dynamic value is
evaluated once when the conversion is reached and checked at runtime. An invalid
value causes a [panic](../error-handling/panic.md) naming the value and both
inclusive bounds. Conversion does not return `Option` or `Result`.

An integer never implicitly narrows to a subrange, including initializers,
assignments, arguments, and return values. Write `Percent(Value)` explicitly.
Local [type inference](../basics/local-variables.md#type-inference) accepts
these conversions, for example `const Level := Percent(50);`.

## Membership

`Value in Percent` tests whether an integer or subrange value lies between the
inclusive bounds and returns `boolean`. The right operand names a subrange type,
including a type alias or imported qualified name such as `Domain.Percent`.
A subrange value on the right is invalid. Real, Boolean, string, enum, and
distinct values on the left are invalid.

The value is evaluated exactly once when the expression is reached. Values
outside the bounds return `false`; the test performs no conversion and does not
panic for an out-of-range value. Normal Boolean precedence and short-circuit
rules apply. With a compile-time integer value, membership is a compile-time
constant expression.

```pascal id=example-3 fragment context=declarations
type Percent = 0..100;

function IsPositivePercent(Input: integer): boolean;
begin
  return Input in Percent and Percent(Input) > 0;
end function;
```

For `Input = 101`, membership returns `false` and `and` skips the conversion.
Membership does not change the input's type: even inside a successful guard,
write the explicit checked conversion `Percent(Input)`.

## Compatibility and operations

Subranges with the same evaluated lower and upper bounds are the same type,
even when declared with different names or in different units. A
[type alias](type-aliases.md) preserves those bounds. Different bounds require
an explicit checked conversion, even when one range contains the other.
Subranges restrict values; [distinct types](distinct-types.md) provide separate
domain identities.

A scalar subrange value widens implicitly to `integer`. Arithmetic and
comparisons use its integer value. Unary negation and binary `+`, `-`, `*`, `div`,
and `mod` produce `integer`; `/` produces `real` under the existing division rule.
Arithmetic does not preserve the bounds. Logical operators still require
Boolean operands.

Scalar widening also applies to array and string indices, integer dictionary
keys, and value membership in integer arrays and dictionaries. A dictionary or
array with a subrange component still requires a value of that subrange type.

```pascal id=example-4 fragment context=subrange-values
const Level: Percent := Percent(50);
const Raw: integer := Level;
const Sum: integer := Level + 60;
const Half: real := Level / 2;
const Changed: Percent := Percent(Level + 10);
```

Whole arrays, dictionaries, generic containers, and callable signatures retain
their exact component types. An `array of Percent` does not become an
`array of integer`. [`var` parameters](../functions/var-parameters.md) require the
same type, including equal bounds, because writes affect the caller's storage.

Subranges satisfy `Comparable` and `Printable` [generic constraints](generics.md),
but not `Numeric`: an arithmetic result cannot promise the original bounds.

## Cases and units

A subrange `case` must explicitly cover every value within the declared bounds,
in statements and expressions alike. `else` is forbidden. Labels and range
endpoints must be compile-time constants; integer labels and checked constant
conversions are accepted. The compiler combines inclusive intervals without
enumerating their values, including inside nested patterns. Extending the bounds
reveals uncovered values at cases that listed only the former domain.

```pascal id=example-5 fragment context=subrange-case
const Category: string := case Level of
  when 0..49: 'low';
  when 50..100: 'high';
end case;
```

Here `Level: Percent` is covered completely. Labels outside the declared domain
and empty ranges match no value and are rejected. See
[exhaustiveness](../pattern-matching/exhaustiveness.md).

Public subrange types and constants retain their bounds across compiled units,
aliases, and generic signatures. Qualified conversions such as `Domain.Percent(Value)`
and membership tests such as `Value in Domain.Percent` use the imported bounds.
