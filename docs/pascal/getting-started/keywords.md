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
div       mod
true      false     type      record
enum      array     channel   task
panic     break     continue  result
option    ok        error     some
none      try       public    go
dict      with      static    property
event     read      write     comparable
numeric   printable self      nil
as        elsif     when      null
discard   equatable
```

Every word in the table is fully reserved, including after `.` in a qualified name or
member access. Some keywords are valid only in their dedicated syntax positions: `read`
and `write` introduce property or event accessors, the four constraint keywords follow
a generic type parameter, and `self` names the first receiver parameter and receiver
expression of an instance record method.

`private` is not a keyword. Unit declarations and record members without
`public` are private by default, so `private` remains available as an ordinary
identifier.

Reserved words cannot be used as declarations or member names, even after a
qualifier. Public APIs must therefore use an identifier-safe spelling such as `EndKey`
instead of `End`, `NoCommand` instead of `None`, or `CompletedCommand` instead of
`Result`. The standard units follow the same rule, which is why they are named `Std.Results` and
`Std.Tasks` rather than after the `result` and `task` keywords. FPAS has no escaped-identifier syntax.

`discard Expression;` explicitly consumes an ignored value; see
[first-class callables](../language/functions/first-class.md#result-consumption-and-discard).

## Example

Keywords and identifiers are case-insensitive:

```pascal
program KeywordDemo;

uses Std.Console as Console;

begin
  Console.WriteLn('same keywords, different casing');
end program;
```

## See also

- [Overview](overview.md)
- [Basics](../language/basics/README.md)

`shl` and `shr` are ordinary identifiers. Integer bit operations use [Std.Bits](../std/numeric/bits.md).
