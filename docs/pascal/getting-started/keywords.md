# Keywords

All keywords are case-insensitive, following traditional Pascal convention:

```text
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
distinct  requires  ensures   scope
equatable using
```

Every word in the table is fully reserved, including after `.` in a qualified name or
member access. The generic constraints are `Equatable`, `Comparable`, `Numeric`,
`Printable`, and `Error`. The first four have no other syntax role; `Error` also constructs
and matches a Result failure and declares an error family.
The keyword `self` names the first receiver parameter and receiver expression of
an instance record method, and `distinct` introduces a
[distinct type](../language/types/distinct-types.md) body after `type Name =`.

`requires` introduces a precondition and `ensures` a postcondition after a named
routine heading. Both are reserved; see [contracts](../language/functions/contracts.md).

`scope` opens a [task scope](../language/concurrency/task-scopes.md) that owns
the tasks started inside it.

`using const` declares a [resource local](../language/basics/local-variables.md#resource-locals).
`release` remains an ordinary identifier except before `procedure` in a
[record release declaration](../language/types/records.md#resource-release).

`event`, `nil`, `read`, `write`, and `Assigned` are ordinary identifiers.
Their declarations and references follow normal case-insensitive name resolution.

`mutable` is an ordinary identifier. Binding declarations use `const` or
`var`, and routine parameters are read-only unless declared with
[`var`](../language/functions/var-parameters.md).

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

```pascal id=example-2
program KeywordDemo;

uses Std.Console as Console;

begin
  Console.writeln('same keywords, different casing');
end.
```

## See also

- [Overview](overview.md)
- [Basics](../language/basics/README.md)

`shl` and `shr` are ordinary identifiers. Integer bit operations use
[Std.Math.Bits](../std/numeric/bits.md); logical operators require boolean operands.
