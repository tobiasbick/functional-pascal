# Generic tree

This example maps a recursive Tree<int> to Tree<string>, matches all union cases,
and stores its rendering in an immutable Summary<string> record. The record has a
computed closure, a default label, and a with update that preserves its other fields.
All bindings, payloads, routine parameters, and results have explicit types.
Map/Count calls infer their arguments, and union constructors infer from payloads
and the expected type.

After the [local SDK bootstrap](../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/generic-tree/GenericTree.fpasproj
dotnet build examples/generic-tree/GenericTree.fpasproj --no-restore
dotnet examples/generic-tree/bin/Debug/net10.0/GenericTree.dll
```

Expected output:

```text
(21, 34)
leaves: 2
True
```

See [generic data types](../../docs/pascal/language/types/generics.md) for constraints,
CLR representations and cross-assembly behavior; [call inference](../../docs/pascal/language/functions/type-inference.md)
describes when explicit type arguments remain necessary.
