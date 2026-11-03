# Hello world

A program imports Console explicitly and writes through the actual .NET API.

```pascal
program HelloWorld;

uses System.Console as Console;

begin
  Console.WriteLine('Hello, world!');
end;
end program;
```

Save sources as UTF-8 .fpas files. The entry block ends with end;, then end program;
closes the declaration. Keywords and identifiers are case-sensitive.

To run a project, follow the [local SDK workflow](../../build/development.md#build-an-fpas-application).
The repository's examples/hello project contains the required .fpasproj file.
Its built DLL runs with dotnet. A standalone source can also be compiled through
the [direct compiler](../program-structure/cli.md).

## See also

- [First program](first-program.md)
- [Primitive types](../language/basics/primitive-types.md)
- [Programs](../program-structure/programs.md)
