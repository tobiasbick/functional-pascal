# Read a file through .NET

ReadFile.fpasproj copies sample.txt into the output directory. main.fpas locates
it through System.AppContext.BaseDirectory, opens System.IO.StreamReader, reads
the text, and disposes the reader through a blockless using const declaration.
An IOException is handled through ordinary CLR catch semantics; finally runs
after the reader's cleanup.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet build examples/io/read-file/ReadFile.fpasproj
dotnet examples/io/read-file/bin/Debug/net10.0/ReadFile.dll
```

Expected output:

```text
Hello from a .NET file.
Read attempt finished
```

The program works regardless of the current working directory. The SDK test
builds these exact sources/project and checks the output. Use a fresh SDK
package version/cache entry after changing packaged compiler tools, as described
in the bootstrap instructions.
