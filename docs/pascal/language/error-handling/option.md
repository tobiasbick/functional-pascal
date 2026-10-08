# Option

`Option of T` represents a value that may be absent:

```pascal
const O: Option of integer := Some(42);
const N: Option of integer := None;
```

## Using Option

```pascal


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

```pascal
const Idx: option of integer := FindIndex([10, 20, 30], 20);
case Idx of
  when Some(const I):
    WriteLn('Found at ' + IntToStr(I));
  when None:
    WriteLn('Not found');
end case;
```

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Option operations`](../types/option-operations.md)
