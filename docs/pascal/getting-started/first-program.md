# First program

A complete program imports its dependencies, declares typed routines, and then
executes one entry block. Public reusable declarations belong in units.

```pascal
program Hello;

uses System.Console as Console;

function Greeting(Name: string): string;
begin
  return 'Hello, ' + Name;
end function;

begin
  Console.WriteLine(Greeting('CLR'));
end;
end program;
```

Public declarations belong in units. Values, parameters, function results, and
record fields have explicit types. Generic argument inference does not permit
omitting a value binding's type. See
[program structure](../program-structure/README.md).

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
