# Keywords

All keywords are case-insensitive, following traditional Pascal convention:

```
program   unit      uses      const
var       function  procedure
begin     end       return    if
then      else      case      of
for       to        downto    in
do        while     repeat    until
and       or        not       xor
div       mod
true      false     type      record
enum      array     channel   task
panic     break     continue  result
option    ok        error     some
none      try       public    go
dict      with      static    numeric
comparable printable self     elsif
when      null      discard   is
```

Every word in the table is fully reserved, including after `.` in a qualified name or
member access. Some keywords are valid only in their dedicated syntax positions:
the three constraint keywords follow a generic type parameter, and `self` names
the first receiver parameter and receiver expression of an instance record method.

`event`, `nil`, `read`, `write`, and `Assigned` are ordinary identifiers.
Their declarations and references follow normal case-insensitive name resolution.

`mutable` is an ordinary identifier. Binding declarations use `const` or
`var`, and routine parameters are read-only.

`private` is not a keyword. Unit declarations and record members without
`public` are private by default, so `private` remains available as an ordinary
identifier.

The reserved words `elsif`, `when`, and `null` also cannot be used as names.
An identifier error suggests a replacement such as `Timestamp` or `NullValue`;
the JSON null constructor is `JsonValue.NullValue`. Longer identifiers such as
`WhenValue` remain valid. Keywords inside strings and comments retain their text.

Reserved words cannot be used as declarations or member names, even after a
qualifier. Public APIs must therefore use an identifier-safe spelling such as `EndKey`
instead of `End`, `NoCommand` instead of `None`, or `CompletedCommand` instead of
`Result`. Standard units follow the same rule; `Std.Tasks` uses an identifier-safe unit name. FPAS has no escaped-identifier syntax.

## Example

Keywords and identifiers are case-insensitive:

```pascal
PROGRAM KeywordDemo;

USES Std.Console;

BEGIN
  writeln('same keywords, different casing');
END.
```

## See also

- [Overview](overview.md)
- [Basics](../language/basics/README.md)

`shl` and `shr` are ordinary identifiers. Integer bit operations use
[Std.Bits](../std/numeric/bits.md); logical operators require boolean operands.
