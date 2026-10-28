# Basics

Read this when declaring values, writing expressions and control flow, or using
native collections. Start with the [entry page](README.md) for program shape.

## Values and bindings

The primitive types are signed 64-bit `integer`, finite binary64 `real`,
`boolean`, and Unicode `string`. There is no separate character type.
Decimal integers permit single underscores between digits; hexadecimal starts
with `$`. Reals need digits on both sides of the decimal point, such as `1.5e2`.
`#10` denotes a Unicode scalar and concatenates with adjacent string literals.

Use `const Name: Type := Value;` or `var Name: Type := Value;`. Locals appear as
statements after `begin`; a nested `begin ... end;` creates a lexical block.
Local types are unavailable. Globals and other declarations outside statement
lists require explicit types. Local inference accepts calls, operators,
field/index reads, closures with complete signatures, and decision expressions.
Empty collections, `None`, and incomplete Result constructors need type context.
The inferred type stays fixed; integer and real do not merge through inference.

`const` means an immutable binding, not necessarily a compile-time value.
Its initializer executes when reached. Case labels and subrange bounds require
compile-time constants.

## Expressions

Assignment is `:=`; equality and inequality are `=` and `<>`. Arithmetic operands
need the same numeric type; convert with `Std.Math.IntToReal` before mixing them.
`/` returns real even for integer operands. Integer-only `div` truncates toward
zero; `mod` follows the dividend's sign. Integer overflow, division by zero and
nonfinite real results fail at runtime. Boolean conditions require booleans.

Precedence from high to low is postfix operations, unary minus/`try`,
`* / div mod`, `+ -`, comparisons/`in`/`is`, `not`, then `and or xor`.
Comparisons do not chain. Parenthesize mixtures of logical operators:
`(A and B) or C`. `and` and `or` short-circuit; `xor` evaluates both operands.
Integer bit operations come from `Std.Math.Bits`.

`if` statements can omit `else`. An `if` expression requires both branches:
`const Sign := if N > 0 then 'positive' else 'other' end if;`.
An expression's closer gets its terminator from the surrounding statement.
Use [case patterns](patterns-errors.md) when selecting or extracting variants.

## Loops and collections

Counting `for` loops include both bounds and declare a typed read-only variable.
`for Value: T in Items do` visits array values; dictionary iteration visits keys
in insertion order. The collection expression is evaluated once. Its writable
storage root cannot be changed during the loop, including through helpers,
`var` arguments or `InPlace` calls. Iterate an independent copy when changing
the original. `while` tests before the body; `repeat` tests after it.
`break;` exits the innermost loop; `continue;` starts its next iteration.

Arrays use `array of T` and `[A, B]`; dictionaries use `dict of K to V` and
`[Key: Value]`. Empty literals are `[]` and `[:]`. Records and collections have
value semantics, including when copied. Handles such as channels still refer
to their shared resource. An invalid array index or missing dictionary key fails
at runtime; use `Lookup.Get(Key)` for optional lookup.

Native dot operations need no imports: `Items.Length()`, `Items.Map(F)`,
`Text.Split(',')`. Transformations preserve the receiver unless named `InPlace`.
`Push` returns a new array; `PushInPlace` changes a writable array. `Pop` returns
`ArrayPopResult of T` with `Remaining` and `Value`; `PopInPlace` returns the removed
element. Both fail on empty arrays. InPlace operations retain the receiver's
static type. String elements themselves are read-only.

```pascal id=loops-and-values run
program AgentBasics;

uses Std.Test as Test;

begin
  var Items := [1, 2];
  const Before := Items;
  Items.PushInPlace(3);
  var Total := 0;
  for Value: integer in Items do
    Total := Total + Value;
  end for;

  for I: integer := 2 downto 1 do
    Total := Total + I;
  end for;

  while Total < 10 do
    Total := Total + 1;
  end while;

  repeat
    Total := Total - 1;
  until Total = 9;
  Test.AssertEquals(9, Total);
  Test.AssertEquals(2, Before.Length());
  Test.AssertTrue(3 in Items);
  Test.AssertTrue(Items.IndexOf(3) = Some(2));
  const Scores: dict of string to integer := ['red': 2];
  Test.AssertTrue(Scores.Get('missing') = None);
  Test.AssertEquals('😀', 'aé😀'[2]);
end.
```

Use `in` for array elements, dictionary keys, string substrings and subrange
membership. Array `IndexOf`/`FindIndex` and string `IndexOf`/`LastIndexOf` return
`Option of integer`; string positions count Unicode scalars. `Sort` is stable
and ascending and needs Comparable elements. `SortBy(F)` permits record elements
when F returns Comparable keys. It computes each key once in input order.

For details, read [operators](../language/basics/operators.md),
[for-in](../language/control-flow/for-in.md),
[arrays](../language/types/array/README.md),
[dictionaries](../language/types/dictionaries.md), or
[strings](../language/types/string/README.md). For a particular operation, use
`fpas doc`, for example `fpas doc string.Format` or `fpas doc array.SortBy`.
