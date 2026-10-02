# Option

`Option of T` represents a value that may be absent:

```pascal
var O: option of integer := Some(42);
var N: option of integer := None;

```

## Using Option

```pascal
function FindIndex(Items: array of integer; Target: integer): option of integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
    end if;
  end for;

  return None;
end function;

```

## Handling with case

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

var Idx: option of integer := FindIndex([10, 20, 30], 20);
case Idx of
  when Some(I):
    Console.WriteLn('Found at ' + Conv.IntToStr(I));
  when None:
    Console.WriteLn('Not found');
end case;
```

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Options`](../../std/result/option.md)
