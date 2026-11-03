# Files and streams

Use `System.IO` for files, directories, paths, readers, and streams. Import the
namespace with an alias and call the .NET API directly. Use `Path.Combine` for
paths and `File.ReadAllText` for a small file that fits in memory; choose a reader
or stream when processing incrementally.

Readers and streams own resources. Bind them with `using const` so disposal
occurs when the enclosing scope exits, including through an exception. Catch the
specific failure types the application can handle.

## Example

```pascal
program ReadWorld;

uses System as Sys, System.IO as IO, System.Console as Console;

begin
  try
    using const Reader: IO.StreamReader := new IO.StreamReader('world.json');
    Console.WriteLine(Reader.ReadToEnd());
  catch Failure: IO.IOException do
    Console.WriteLine(Failure.Message);
  catch Failure: Sys.UnauthorizedAccessException do
    Console.WriteLine(Failure.Message);
  finally
    Console.WriteLine('Read attempt finished');
  end try;
end;
end program;
```

The file must exist for the success path. The acquired reader is disposed even
when reading fails. Failure to construct it does not dispose an unacquired reader.
The declaration belongs to the try-body scope, so disposal occurs before any
catch/finally body executes.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
