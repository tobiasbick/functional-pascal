# Option

`Option of (T)` represents a value that may be absent:

```pascal
var O: option of (integer) := Option.Some(42);
var N: option of (integer) := Option.None;

```

## Using Option

```pascal
uses Std.Arrays as Arrays;

function FindIndex(Items: array of (integer); Target: integer): option of (integer);
begin
  for I: integer := 0 to Arrays.Length(Items) - 1 do
    if Items[I] = Target then
      return Option.Some(I);
    end if;
  end for;

  return Option.None;
end function;

```

## Handling with case

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

var Idx: option of (integer) := FindIndex([10, 20, 30], 20);
case Idx of
  when Option.Some(const I):
    Console.WriteLn('Found at ' + Conv.IntToStr(I));
  when Option.None:
    Console.WriteLn('Not found');
end case;
```

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Options`](../../std/result/option.md)
