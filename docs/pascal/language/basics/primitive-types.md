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

There are no implicit boxing-based collection conversions or implicit narrowing
numeric conversions. Ordinary CLR boxing to `object` or an implemented interface
is supported. Explicit unboxing uses `cast<T>` and requires the actual boxed type.

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
