# Programs

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

A program has a qualified name, optional imports, declarations, and one entry
body. An executable compilation has exactly one program; a library has none.
The entry body is a `begin ... end;` block. A separate `end program;` closes the
program declaration, matching the unit's named `end unit;` terminator. Both the
body's `end;` and the declaration's `end program;` are required for synchronous
and async programs. A terminal period cannot replace either terminator. Routines
retain their own named terminators.
`async program Name;` enables await in the entry body. Source selection and build
configuration are specified in the [MSBuild contract](../../build/msbuild.md).

The entry body receives an implicit immutable `args: array of string` binding, containing
command-line arguments excluding the executable name. Free functions do not
implicitly receive args. Entry completion yields process exit code zero;
unhandled exceptions follow the .NET host's failure behavior. The baseline has
no value-returning top-level entry body.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
