# Values, expressions, and control flow

Read this when writing bindings, calculations, loops, arrays, or text processing.
The [entrypoint](README.md) gives the complete source shape.

## Bindings and literals

Every declaration has one name, an explicit type, and an initializer:
`const Limit: int := 10;` or `var Count: int := 0;`. Repeat the declaration
keyword for each binding. A const initializer can run arbitrary code; const
does not mean a compile-time constant. Assignment uses `:=`.

Primitive aliases denote real CLR types: bool; byte/sbyte; short/ushort; int/uint;
long/ulong; nint/nuint; float/double/decimal; char; string; object. Signed int is
32-bit, long is 64-bit, and native integers follow the execution architecture.
Use an explicit conversion rather than guessing a different Pascal dialect's type.

Strings use apostrophes: `'it''s ready'`. Numeric forms include `$FF`, `1_000`,
`42L`, `42U`, `42UL`, `1.5`, `1e3`, `1.5F`, and `1.5M`. Unsuffixed integers must
fit int; representable constants can convert contextually to appropriate smaller/
unsigned types. `#65` is char. Strings cannot span lines; concatenate with `#10`
for a line feed. Only `//` starts a comment. Whitespace and comments do not
replace required statement terminators.

## Operators and decisions

Arithmetic follows CLR/C# numeric promotion: short plus short produces int,
and int plus long produces long. Decimal does not implicitly mix with binary
floating-point types. Integer arithmetic/conversions check overflow; use
`unchecked(Expression)` for wrapping integral operations. Decimal retains checks.
Floating operations can produce NaN/infinity. Subrange validation always remains.

Use div/mod for integral division/remainder. `/` requires floating/decimal
operands; cast first when starting with integers. Casts use `cast<T>(Value)`.
No implicit conversion to bool exists. Boolean and/or short-circuit; xor is eager.
Integral and/or/xor/not and shl/shr are bitwise, with CLR promotion/shift rules.

Precedence, highest first: postfix operations; unary +, -, not, await;
multiplication/division; addition/subtraction; shifts; ordering/is; equality;
and; xor; or; right-associative ??. Comparisons do not chain. Parenthesize when
the intended grouping is unclear. String + accepts strings/chars, not arbitrary
object formatting. Use actual .NET formatting APIs for other values.

If statements use if/then, optional elsif/then, optional else, and end if.
An if expression requires else and one compatible expression per branch.
Every statement ends with a semicolon, including the statement before else.
Nested begin/end blocks, branches, and loop bodies introduce local scopes.

## Arrays, strings, and loops

Vectors use int[]; rectangular arrays use int[,]; jagged arrays use int[][].
Literals such as [1, 2] produce vectors; empty [] needs an expected element type.
Allocate with new int[Count] or new double[Rows, Columns]. Length is a property,
and indexes start at zero. Nonempty default-initialized reference arrays require
nullable elements, such as new string?[Count]. Use initialized literals for
non-null elements. Arrays are mutable references even through const bindings.

Strings are immutable UTF-16. Text.Length and Text[Index] count code units;
indexing returns char, so a non-BMP character occupies two indexes. Use
System.Text.Rune and other .NET text APIs for Unicode scalars/graphemes.

Counting loops declare a read-only typed variable and include both bounds.
Bounds evaluate once; reversed ranges are empty; endpoints never overflow while
stepping. Use downto for descending order. For-in requires a typed element binding
and follows the collection's enumeration/mutation rules. Disposable enumerators
are cleaned up on all exits. While tests first; repeat/until tests after body
cleanup. Continue reaches the next iteration/test; break exits the innermost loop.

```pascal
program AgentBasics;

uses System as Sys, System.Console as Console;

begin
  const Items: int[] := [1, 2];
  const Shared: int[] := Items;
  Shared[0] := 3;
  var Total: int := 0;
  for Value: int in Items do Total := Total + Value; end for;
  for Index: int := 1 to 2 do Total := Total + Index; end for;
  repeat Total := Total - 1; until Total = 7;
  while Total < 9 do Total := Total + 1; end while;

  const Text: string := 'A😀';
  const Half: double := cast<double>(Total) / 2.0;
  if Total <> 9 or Items[0] <> 3 or Text.Length <> 3 or Text[0] <> #65 or Half <> 4.5 then
    raise new Sys.InvalidOperationException('Incorrect value, loop, or text behavior');
  end if;
  Console.WriteLine('Basics verified');
end;
end program;
```

Read [basics](../language/basics.md) for literal/conversion/operator details,
[control flow](../language/control-flow.md) for scopes and exit rules, and
[types](../language/types.md#arrays-and-collections) for CLR array contracts.
For collection APIs, read [interop](interop.md); dictionaries/lists are ordinary
.NET objects rather than built-in language collection wrappers.
