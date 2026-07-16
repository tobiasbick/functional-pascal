# Keywords

All keywords are case-insensitive, following traditional Pascal convention:

```
program   unit      uses      const
var       mutable   function  procedure
begin     end       return    if
then      else      case      of
for       to        downto    in
do        while     repeat    until
and       or        not       xor
div       mod       shl       shr
true      false     type      record
enum      array     panic     break
continue  result    option    ok
error     some      none      try
public    private   go        dict
with      static
```

Formal token definitions: [`grammar.ebnf`](../../specs/grammar.ebnf).

## Example

Keywords and identifiers are case-insensitive:

```pascal
PROGRAM KeywordDemo;

BEGIN
  writeln('same keywords, different casing')
END.
```

## See also

- [Overview](overview.md)
- [Basics](../language/basics/README.md)
