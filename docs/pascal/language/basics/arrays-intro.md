# Arrays intro

Arrays are declared with the `array of` syntax:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`array of`, array literals).

```pascal
var
  Numbers: array of integer := [1, 2, 3, 4, 5];
  Names: array of string := ['Alice', 'Bob', 'Charlie'];
  Empty: array of integer := [];
```

Accessing elements uses bracket notation (0-based index):

```pascal
var
  First: integer := Numbers[0];   { 1 }
  Second: string := Names[1];     { 'Bob' }
```

For growth, mutation, and stdlib helpers, see [Arrays](../types/arrays.md).

## See also

- [Arrays](../types/arrays.md)
- [For-in](../control-flow/for-in.md)
