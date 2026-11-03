# Scalar data domains

This executable combines transparent aliases, decimal arithmetic, real CLR enums,
nominal distinct values, and a checked subrange. Every binding has an explicit type.
Port matching covers the complete declared interval without a fallback. A freshly
allocated port array starts at the lower bound, 1.

With the locally packaged SDK available as described in the
[MSBuild milestone](../../docs/implementation/msbuild.md):

```sh
dotnet build examples/data-domains/DataDomains.fpasproj
dotnet examples/data-domains/bin/Debug/net10.0/DataDomains.dll
```

Expected output:

```text
42
Ada
Online
19.95
system
1
```

See the [implemented scalar types](../../docs/implementation/scalar-types.md)
for conversions, CLR signatures, and cross-assembly contracts.
