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
public    go        dict      with
static    property  event
nil
```

`property`, `event`, and `nil` are reserved words. `event` and `property` are also
accepted as ordinary identifiers outside record-member declaration position so existing
type names such as `Std.Console.Event` keep working.

`private` is not a keyword. Unit declarations and record members without
`public` are private by default, so `private` remains available as an ordinary
identifier.

Reserved words generally cannot be used as declarations or member names, even after a
qualifier. Public APIs must therefore use an identifier-safe spelling such as `EndKey`
instead of `End`, `NoCommand` instead of `None`, or `CompletedCommand` instead of
`Result`. FPAS currently has no escaped-identifier syntax.

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
