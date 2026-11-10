# Typed loops

The async program counts through a byte subrange, executes a repeat/until loop,
and reaches ulong.MaxValue without taking an overflowing final step. Continue in
the repeat body still evaluates the until condition.

After [bootstrapping the local SDK](../../../docs/build/msbuild.md):

```sh
dotnet build examples/basics/loops/Loops.fpasproj
dotnet examples/basics/loops/bin/Debug/net10.0/Loops.dll
```

Expected output:

```text
1
2
3
repeat: 3
18446744073709551614
18446744073709551615
```

See [typed loops](../../../docs/pascal/language/control-flow/loops.md) for scope, cleanup, and
checked counter rules.
