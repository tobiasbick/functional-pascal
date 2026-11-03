# Generic pipeline

This executable example defines generic Identity, Map, and Compose functions in
a unit. Compose returns a typed closure, Map enumerates a real .NET IEnumerable
and collects results in List<U>, and Identity<string> becomes a delegate.
Every declaration keeps its explicit value/parameter/result types. Compose and
Map infer their arguments from supplied callables and values; generic method
groups such as Identity<string> retain explicit arguments.

After building and locally packaging the compiler as described in
[MSBuild integration](../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/generic-pipeline/GenericPipeline.fpasproj
dotnet build examples/generic-pipeline/GenericPipeline.fpasproj --no-restore
dotnet examples/generic-pipeline/bin/Debug/net10.0/GenericPipeline.dll
```

Expected output:

```text
Value: 2
Value: 4
Value: 6
identity
```

Map uses a List<U> because allocating a nonempty U[] would leave uninitialized
null elements when U is a non-nullable reference type. The compiler enforces this
rule even when U has not yet been instantiated.
See [generic routines](../../docs/pascal/language/functions/generic-routines.md) for
constraints, nullable semantics, CLR interop, and remaining limits.
