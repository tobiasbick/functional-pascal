# Named local functions

Factorial calls a local function declared later in its lexical block. Multiply
recurses directly. MakeCounter returns a local function as a CLR delegate; its
shared captured cell survives the enclosing call.

After [bootstrapping the local SDK](../../docs/implementation/msbuild.md):

```sh
dotnet build examples/local-functions/LocalFunctions.fpasproj
dotnet examples/local-functions/bin/Debug/net10.0/LocalFunctions.dll
```

Expected output:

```text
120
11
12
```

See [nested routines](../../docs/implementation/nested-routines.md) for scope,
capture initialization, overloads, generics, parameter modes, and async behavior.
