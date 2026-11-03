# Scalar data domains

This executable combines transparent aliases, decimal arithmetic, real CLR enums,
nominal distinct values, and a checked subrange. Every binding has an explicit type.
Port matching covers the complete declared interval without a fallback. A freshly
allocated port array starts at the lower bound, 1.

With the locally packaged SDK available as described in the
[development workflow](../../docs/build/development.md#build-an-fpas-application):

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

See [data domains](../../docs/pascal/language/types/data-domains.md) for wrapping,
bounds, and cross-assembly use, and [CLR mapping](../../docs/specs/clr-mapping.md)
for emitted signatures and metadata contracts.
