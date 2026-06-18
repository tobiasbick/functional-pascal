# Scalar labels

## Basic matching

```pascal
case Value of
  1: WriteLn('one');
  2: WriteLn('two');
  3: WriteLn('three');
else
  WriteLn('other');
end;
```

## Multiple values

Separate multiple values with commas. Every label in the list shares the same arm body (and the same pattern bindings when applicable):

```pascal
case Day of
  'Monday':    WriteLn('Start of week');
  'Friday':    WriteLn('Almost weekend');
  'Saturday',
  'Sunday':    WriteLn('Weekend');
else
  WriteLn('Midweek');
end;
```

## Else branch

Use `else` to handle all remaining cases:

```pascal
case L of
  Light.Red:  WriteLn('Stop');
else
  WriteLn('Proceed with caution');
end;
```

## Block arms

Use `begin..end` when a case arm needs multiple statements:

```pascal
case Command of
  'help':
    begin
      WriteLn('Available commands:');
      WriteLn('  help, quit, run');
    end;
  'quit':
    WriteLn('Goodbye');
else
  WriteLn('Unknown command');
end;
```

## See also

- [Ranges](ranges.md)
- [Guards](guards.md)
