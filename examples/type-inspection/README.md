# Runtime type identity

This program obtains CLR Type objects for primitive, nullable, array, source
record, and generic parameter types without constructing instances.

After the [local SDK bootstrap](../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/type-inspection/TypeInspection.fpasproj
dotnet build examples/type-inspection/TypeInspection.fpasproj --no-restore
dotnet examples/type-inspection/bin/Debug/net10.0/TypeInspection.dll
```

Expected output:

```text
System.Int32
True
True
2
String
```
