# Primitive types

These aliases denote the actual CLR types, not wrappers.

| Alias | CLR type |
| --- | --- |
| `bool` | `System.Boolean` |
| `byte`, `sbyte` | `System.Byte`, `System.SByte` |
| `short`, `ushort` | `System.Int16`, `System.UInt16` |
| `int`, `uint` | `System.Int32`, `System.UInt32` |
| `long`, `ulong` | `System.Int64`, `System.UInt64` |
| `nint`, `nuint` | `System.IntPtr`, `System.UIntPtr` |
| `float`, `double`, `decimal` | `System.Single`, `System.Double`, `System.Decimal` |
| `char` | `System.Char`, one UTF-16 code unit |
| `string`, `object` | `System.String`, `System.Object` |

Procedures return `System.Void`, which is not a general source value type.
Native-sized integers follow the target architecture. `string` is immutable;
its `Length` and indexes use UTF-16 code units. `Text[Index]` returns `char`.
Use imported .NET APIs for Unicode scalars, graphemes, encoding, and formatting.

There are no implicit boxing-based collection conversions. Implicit narrowing
numeric conversions are limited to the representable constant expressions
described in [literal rules](lexical-rules.md#literals). Ordinary CLR boxing to
`object` or an implemented interface is supported. Explicit unboxing uses
`cast Value as T` and requires the actual boxed type.

## Example

```pascal
program PrimitiveValues;

uses System.Console as Console;

begin
  const Name: string := 'Alpha';
  const SampleSize: int := 20;
  const SampleCount: long := 4_000_000_000L;
  const Zoom: double := 1.5;
  const Price: decimal := 12.50M;
  const Initial: char := #77;
  const Ready: bool := true;

  Console.WriteLine('Hello, ' + Name);
  Console.WriteLine(SampleSize);
  Console.WriteLine(SampleCount);
  Console.WriteLine(Zoom);
  Console.WriteLine(Price);
  Console.WriteLine(Initial);
  Console.WriteLine(Ready);
end;

end program;
```

There are no predefined integer/real aliases. Library properties have their
actual names: `'Hello'.Length`, not `Length()`.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

## Source files

<details>
<summary>Browse 13 source files</summary>

- `src/Fpas.Compiler/Binding/Bodies/`
  - [Binder.Numbers.cs](../../../../src/Fpas.Compiler/Binding/Bodies/Binder.Numbers.cs)
- `src/Fpas.Compiler/Binding/Bound/`
  - [BoundNumber.cs](../../../../src/Fpas.Compiler/Binding/Bound/BoundNumber.cs)
  - [ConstantValue.cs](../../../../src/Fpas.Compiler/Binding/Bound/ConstantValue.cs)
- `src/Fpas.Compiler/Binding/Types/`
  - [NumericFacts.cs](../../../../src/Fpas.Compiler/Binding/Types/NumericFacts.cs)
  - [NumericValue.cs](../../../../src/Fpas.Compiler/Binding/Types/NumericValue.cs)
- `src/Fpas.Compiler/Emission/Bodies/`
  - [BodyEmitter.Numbers.cs](../../../../src/Fpas.Compiler/Emission/Bodies/BodyEmitter.Numbers.cs)
- `tests/Fpas.Compiler.Tests/ControlFlow/`
  - [NativeConstantTests.cs](../../../../tests/Fpas.Compiler.Tests/ControlFlow/NativeConstantTests.cs)
- `tests/Fpas.Compiler.Tests/Types/`
  - [ScalarDiagnosticsTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/ScalarDiagnosticsTests.cs)
  - [ScalarMetadataTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/ScalarMetadataTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.Scalars.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Scalars.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [NumericExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/NumericExecutionTests.cs)
  - [PrimitiveExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/PrimitiveExecutionTests.cs)
  - [ScalarExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/ScalarExecutionTests.cs)

</details>
