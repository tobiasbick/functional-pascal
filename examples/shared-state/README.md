# Shared unit state

The unit exposes a readonly limit, a mutable counter, and a delegate reading the
current counter. The program calls unit routines and passes the global field by
ref. All fields initialize once through the unit's CLR type initializer.

After [bootstrapping the local SDK](../../docs/build/msbuild.md):

```sh
dotnet build examples/shared-state/SharedState.fpasproj
dotnet examples/shared-state/bin/Debug/net10.0/SharedState.dll
```

Expected output:

```text
3
1000
0
```

See [global values](../../docs/pascal/program-structure/units.md) for visibility,
initialization, nullable values, and contract rules. Shared field mutation needs
ordinary .NET synchronization when accessed concurrently.
