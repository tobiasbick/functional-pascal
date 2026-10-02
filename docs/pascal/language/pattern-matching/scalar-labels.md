# Scalar labels

## Basic matching

```pascal
uses Std.Console as Console;

case Value of
  when 1:
    Console.WriteLn('one');
  when 2:
    Console.WriteLn('two');
  when 3:
    Console.WriteLn('three');
  else
    Console.WriteLn('other');
end case;
```

## Multiple values

Separate multiple values with commas. Every label in the list shares the same arm body (and the same pattern bindings when applicable):

```pascal
uses Std.Console as Console;

case Day of
  when 'Monday':
    Console.WriteLn('Start of week');
  when 'Friday':
    Console.WriteLn('Almost weekend');
  when 'Saturday', 'Sunday':
    Console.WriteLn('Weekend');
  else
    Console.WriteLn('Midweek');
end case;
```

## Else branch

Use `else` to handle all remaining cases:

```pascal
uses Std.Console as Console;

case L of
  when Light.Red:
    Console.WriteLn('Stop');
  else
    Console.WriteLn('Proceed with caution');
end case;
```

## Block arms

An arm holds a statement list, so multiple statements need no `begin` wrapper.
Each `when` arm and the `else` body has its own local declaration scope. Names
declared there are unavailable in neighboring arms or after `end case;`. An
explicit `begin ... end;` block creates a further nested scope:

```pascal
uses Std.Console as Console;

case Command of
  when 'help':
    begin
      Console.WriteLn('Available commands:');
      Console.WriteLn('  help, quit, run');
    end;
  when 'quit':
    Console.WriteLn('Goodbye');
  else
    Console.WriteLn('Unknown command');
end case;
```

## See also

- [Ranges](ranges.md)
- [Guards](guards.md)
