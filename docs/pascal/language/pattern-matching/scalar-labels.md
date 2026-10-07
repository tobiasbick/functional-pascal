# Scalar labels

Value labels must be [compile-time constants](../basics/constants.md#compile-time-constants).
Literal values, static constant bindings, and expressions using known operands
are allowed. Calls, variables, and computed `const` bindings are rejected,
including when the arm also has a guard. Put dynamic comparisons in a
[scalar guard binding](guards.md#scalar-guard-bindings).

## Basic matching

```pascal
case Value of
  when 1:
    WriteLn('one');
  when 2:
    WriteLn('two');
  when 3:
    WriteLn('three');
  else
    WriteLn('other');
end case;
```

## Multiple values

Separate multiple values with commas. Every label in the list shares the same arm body (and the same pattern bindings when applicable):

```pascal
case Day of
  when 'Monday':
    WriteLn('Start of week');
  when 'Friday':
    WriteLn('Almost weekend');
  when 'Saturday', 'Sunday':
    WriteLn('Weekend');
  else
    WriteLn('Midweek');
end case;
```

## Else branch

Use `else` to handle all remaining cases:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  else
    WriteLn('Proceed with caution');
end case;
```

## Block arms

An arm holds multiple statements directly. Use an explicit `begin ... end;`
only when those statements need an additional nested scope:

```pascal
case Command of
  when 'help':
    WriteLn('Available commands:');
    WriteLn('  help, quit, run');
  when 'quit':
    WriteLn('Goodbye');
  else
    WriteLn('Unknown command');
end case;
```

## See also

- [Ranges](ranges.md)
- [Guards](guards.md)
