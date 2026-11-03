# Nullable types

- `T?` for a non-nullable value type means `System.Nullable<T>`.
- `T?` for a reference type is a nullable reference annotation; it does not
  construct `Nullable<T>` or a wrapper object.
- `T` reference values are statically non-null. Definite assignment and nullable
  flow analysis diagnose possible null dereferences and invalid assignments.
- Nullable metadata is imported and emitted. Unannotated foreign reference
  results are conservatively nullable; external annotations are contracts, not
  a guarantee that external code cannot return null.
- A null test on a stable local/parameter refines it for the applicable branch
  and the corresponding short-circuit Boolean operand. A mutable local captured
  by a closure, or invalidated by assignment/byref use, is not stable. Reading a
  mutable property twice does not establish non-nullness.
- A foreign overloaded equality operator can define its own null behavior and
  does not establish non-nullness. A runtime `Value is T` check can validate a
  nullable reference independently of such an operator. Generated FPAS record/
  union equality retains its compiler-defined null-check behavior.
- Use a null branch, `??`, or explicit validation to obtain a non-null value.
  There is no null-forgiving `!` operator or implicit conversion to a union.

`null` cannot initialize a non-null reference or non-nullable value type.
Unconstrained generic parameters cannot be suffixed with `?`; use an appropriate
reference/value constraint. Repeated nullable suffixes are invalid.

## Example

```pascal
program NullableText;

uses System.Environment as Environment, System.Console as Console;

begin
  const OptionalName: string? := Environment.GetEnvironmentVariable('FPAS_PLAYER');
  const Name: string := OptionalName ?? 'Explorer';
  Console.WriteLine(Name);

  if OptionalName <> null then
    Console.WriteLine(OptionalName.Length);
  end if;

  const Text: string := 'A😀';
  Console.WriteLine(Text.Length); // 3 UTF-16 code units
  const First: char := Text[0];
  Console.WriteLine(First);       // A
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
