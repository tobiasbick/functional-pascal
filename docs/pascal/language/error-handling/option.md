# Option

`Option of T` represents a value that may be absent:

```pascal
var O: Option of integer := Some(42);
var N: Option of integer := None;
```

## Using Option

```pascal
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
```

## Handling with case

```pascal
var Idx: Option of integer := FindIndex([10, 20, 30], 20);
case Idx of
  Some(I): WriteLn('Found at ' + IntToStr(I));
  None:    WriteLn('Not found');
end;
```

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Option`](../../std/result/option.md)
