# If statements and expressions

Choose branches using bool conditions; only the selected branch executes.

```pascal fragment context=health
if Health <= 0 then
  Console.WriteLine('Defeated');
elsif Health < 5 then
  Console.WriteLine('In danger');
else
  Console.WriteLine('Ready');
end if;
```

Conditions have type `bool`. `elsif` is one keyword. Each statement, including
the one before `elsif` or `else`, ends with `;`. An `if` expression requires
`else` and one expression per branch:

```pascal fragment context=health
const Label: string := if Health > 0 then
  'Alive'
else
  'Defeated'
end if;
```

Branch results must have a common type through identity, expected type, or an
applicable implicit conversion. Incompatible results require explicit conversion.
Conditions and branches execute only as selected.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
