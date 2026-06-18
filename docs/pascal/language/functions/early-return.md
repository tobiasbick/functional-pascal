# Early return

`return` both sets the return value and exits the function immediately:

```pascal
function IndexOf(Items: array of string; Target: string): integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
  begin
    if Items[I] = Target then
      return I;
  end;
  return -1;
end;
```

## See also

- [Declarations](declarations.md)
