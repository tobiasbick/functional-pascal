# Program with multiple units

Units.fpasproj compiles main.fpas and both sources under lib into one assembly.
The default `**/*.fpas` glob selects them. Unit headers define Demo.Numbers and
Demo.Messages; source filenames and glob order do not define their identities.

After the [local SDK bootstrap](../../docs/implementation/msbuild.md#local-sdk-bootstrap):

```sh
dotnet build examples/units/Units.fpasproj
dotnet examples/units/bin/Debug/net10.0/Units.dll
```

Expected output:

```text
55
The expected sum
```

The program imports each unit with an explicit alias and calls its public
functions. Add is private to Demo.Numbers. References between units use signatures
collected before any body is bound, so declaration/file ordering does not impose
a dependency order. No separate DLL or project is needed for each unit.

When changing packaged tools, use a fresh SDK version/cache entry as described
in the bootstrap instructions. Integration tests build these exact checked-in
sources and project with a uniquely versioned current SDK.
