# Mutable parameters

By default, parameters are immutable inside the routine body. Prefix a parameter with `mutable` to allow reassignment to the local binding:

```pascal
procedure Inc(mutable X: integer);
begin
  X := X + 1;
end;
```

`mutable` only affects the local binding — the caller's value is not changed. To observe changes in the caller, pass a reference type (array, record instance) and mutate its contents. See [Records — immutability](../types/records.md#immutability) for details.

## See also

- [Parameters](parameters.md)
