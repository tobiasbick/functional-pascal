# Early return

`return` both sets the return value and exits the function immediately:

```pascal
uses Std.Arrays as Arrays;

function IndexOf(Items: array of string; Target: string): integer;
begin
  for I: integer := 0 to Arrays.Length(Items) - 1 do
    begin
      if Items[I] = Target then
        return I;
      end if;
    end;
  end for;

  return -1;
end function;

```

## See also

- [Declarations](declarations.md)
