# Numbers and random values

Use System numeric types and Math directly. Specify culture when parsing or
formatting machine-readable text, and select a rounding rule explicitly.

## Parse quantities and bound a percentage

This example parses a decimal quantity and shows two rounding strategies for
the same midpoint. A separate helper reads a percentage, rejects non-finite
values, clamps it to zero through one hundred, and rounds to an integer.
The string-based TryParse overloads take NumberStyles, IFormatProvider, and an
explicit out argument. They return false for invalid input; application range
and finiteness checks remain separate.

```pascal
program NumericUtilities;

uses System as Sys, System.Console as Console, System.Globalization as Global;

function ReadPercent(Text: string): int;
begin
  var Value: double := 0.0;
  if not double.TryParse(Text, Global.NumberStyles.Float,
    Global.CultureInfo.InvariantCulture, out Value) or not double.IsFinite(Value) then
    raise new Sys.FormatException('Invalid percentage');
  end if;
  return cast<int>(Sys.Math.Round(Sys.Math.Clamp(Value, 0.0, 100.0), Sys.MidpointRounding.AwayFromZero));
end function;

begin
  const Culture: Global.CultureInfo := Global.CultureInfo.InvariantCulture;
  var Quantity: decimal := 0.0M;
  if not decimal.TryParse('12.345', Global.NumberStyles.AllowDecimalPoint, Culture, out Quantity) then
    raise new Sys.FormatException('Invalid quantity');
  end if;
  const Even: decimal := decimal.Round(Quantity, 2, Sys.MidpointRounding.ToEven);
  const Away: decimal := decimal.Round(Quantity, 2, Sys.MidpointRounding.AwayFromZero);
  if Even <> 12.34M or Away <> 12.35M or ReadPercent('-4.2') <> 0 or
    ReadPercent('110.2') <> 100 or ReadPercent('42.5') <> 43 then
    raise new Sys.InvalidOperationException('Incorrect rounding or bounds');
  end if;
  var Rejected: int := 0;
  for Invalid: string in ['NaN', 'Infinity', '1e5000', 'invalid'] do
    try
      discard ReadPercent(Invalid);
    catch Failure: Sys.FormatException do
      Rejected := Rejected + 1;
    end try;
  end for;
  if Rejected <> 4 then raise new Sys.InvalidOperationException('Invalid percentage accepted'); end if;
  Console.WriteLine(Even.ToString('0.00', Culture));
  Console.WriteLine(Away.ToString('0.00', Culture));
  Console.WriteLine(ReadPercent('42.5'));
end;
end program;
```

The output is `12.34`, `12.35`, and `43`. Decimal keeps these base-ten quantities
separate from binary floating-point arithmetic. The
[Decimal.Round](https://learn.microsoft.com/en-us/dotnet/api/system.decimal.round?view=net-10.0)
overload selects precision and midpoint handling. Math.Clamp bounds a value;
it is not a substitute for Double.IsFinite when accepting floating-point input.
The helper rejects NaN, infinity, and overflow to infinity before clamping.

## Choose a random generator

Use a local seeded Random for replayable simulation within a selected runtime.
[Random's algorithm](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-random)
is not a cross-version sequence contract. Use
[Random.Shared](https://learn.microsoft.com/en-us/dotnet/api/system.random.shared?view=net-10.0)
for a thread-safe shared generator when replay is unnecessary; ordinary Random
instances require synchronization if shared across threads.

For cryptographically strong random integers or bytes, use RandomNumberGenerator.
These APIs are distinct from seeded simulation. The example selects a bounded
index and allocates a byte array through GetBytes(Int32), without Span signatures.
Next(min, max) and GetInt32(min, max) include the lower bound and exclude the upper.

```pascal
program RandomValues;

uses System as Sys, System.Console as Console, System.Security.Cryptography as Crypto;

begin
  const Replay: Sys.Random := new Sys.Random(1729);
  for Index: int := 1 to 3 do Console.WriteLine(Replay.Next(1, 7)); end for;
  const Die: int := Sys.Random.Shared.Next(1, 7);
  const Names: array of string := ['North', 'East', 'West'];
  const Choice: int := Crypto.RandomNumberGenerator.GetInt32(0, Names.Length);
  const Bytes: array of byte := Crypto.RandomNumberGenerator.GetBytes(16);
  if Die < 1 or Die >= 7 or Choice < 0 or Choice >= Names.Length or Bytes.Length <> 16 then
    raise new Sys.InvalidOperationException('Incorrect random range or buffer');
  end if;
  Console.WriteLine('Random values verified');
end;
end program;
```

The first three lines are seeded die rolls; the final line is
`Random values verified`. The Markdown test compares the seeded results with
ordinary C# calls on its current .NET runtime. Unseeded values are checked for
their allowed ranges and buffer size, without probabilistic distribution tests
or a fixed expected byte sequence.

## See also

- [Library guides](README.md)
- [Environment settings and time](environment-and-time.md)
- [Primitive numeric rules](../language/basics/primitive-types.md)
- [RandomNumberGenerator.GetInt32](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.randomnumbergenerator.getint32?view=net-10.0)
- [RandomNumberGenerator.GetBytes](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.randomnumbergenerator.getbytes?view=net-10.0)
