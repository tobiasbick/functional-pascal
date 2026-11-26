# Supported signatures and limits

The initial compiler must reject unsupported metadata with a specific diagnostic.
In particular, it must not pretend that ref structs, pointers, function
pointers, ref returns, unmanaged calling conventions, expression trees, arbitrary
custom awaiters, init-only object initializers, or dynamic binding are ordinary
supported calls. Consuming ordinary managed APIs is the initial interoperability
target; complete C# language and ABI feature coverage is not assumed.

## Span parameters

A call to an imported method or constructor accepts a one-dimensional array, or
an array literal, for a by-value `Span<T>` or `ReadOnlySpan<T>` parameter. The
compiler saves the array with the other arguments and creates the span directly
before the call, so no span value is stored, captured or kept across `await`.
`Span<T>` requires the array's exact element type. `ReadOnlySpan<T>` also accepts
arrays whose elements convert to `T` by reference, and a generic `T` is inferred
from the array element. A candidate that needs fewer span conversions is always
better, so existing array, collection and `params` overloads keep precedence;
among otherwise equal span overloads, `ReadOnlySpan<T>` wins.

Spans remain unsupported everywhere else: as source types, locals, fields,
results, `ref`/`out`/`in` parameters, generic arguments, method-group delegates
and protocol or delegate signatures. Strings and array segments are not
converted to spans; use the API's string or offset/count overloads.

```pascal
program SpanArguments;

uses System.Buffers.Binary as Binary;
uses System.Console as Console;
uses System.IO as IO;

begin
  const Header: array of byte := [$00, $00, $01, $02];
  const Stream: IO.MemoryStream := new IO.MemoryStream();
  Stream.Write(Header);
  Console.WriteLine(Binary.BinaryPrimitives.ReadUInt32BigEndian(Header));
  Console.WriteLine(Stream.Length);
end;

end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)

## Source files

<details>
<summary>Browse 11 source files</summary>

- `src/Fpas.Compiler/Binding/References/`
  - [MetadataConstants.cs](../../../src/Fpas.Compiler/Binding/References/MetadataConstants.cs)
  - [ReferenceCatalog.SignatureLimits.cs](../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.SignatureLimits.cs)
  - [ReferenceCatalog.SourceTypes.cs](../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.SourceTypes.cs)
  - [ReferenceCatalog.Types.cs](../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.Types.cs)
  - [ReferenceSymbols.cs](../../../src/Fpas.Compiler/Binding/References/ReferenceSymbols.cs)
  - [SignatureTypes.cs](../../../src/Fpas.Compiler/Binding/References/SignatureTypes.cs)
- `tests/Fpas.Compiler.Tests/ControlFlow/`
  - [InteropDiagnosticsTests.cs](../../../tests/Fpas.Compiler.Tests/ControlFlow/InteropDiagnosticsTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.Interop.cs](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Interop.cs)
  - [MsBuildTests.SignatureLimits.cs](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.SignatureLimits.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [InteropExecutionTests.cs](../../../tests/Fpas.IntegrationTests/Execution/InteropExecutionTests.cs)
  - [RectangularInteropTests.cs](../../../tests/Fpas.IntegrationTests/Execution/RectangularInteropTests.cs)

</details>
