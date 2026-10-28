# Option

The code fragments use `uses Std.Text.Format as Conv, Std.Console as Console;`.

`Option of T` represents a value that may be absent:

```pascal id=example-1 fragment context=body
const O: option of integer := Some(42);
const N: option of integer := None;
```

## Using Option

```pascal id=example-2 fragment context=declarations
function FindIndex(Items: array of integer; Target: integer): option of integer;
begin
  for I: integer := 0 to Items.Length() - 1 do
    if Items[I] = Target then
      return Some(I);
    end if;
  end for;

  return None;
end function;
```

## Handling with case

```pascal id=example-3 fragment context=option-find
const Idx: option of integer := FindIndex([10, 20, 30], 20);
case Idx of
  when Some(const I):
    Console.WriteLn('Found at ' + Conv.IntToStr(I));
  when None:
    Console.WriteLn('Not found');
end case;
```

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Option operations`](../types/option-operations.md)
