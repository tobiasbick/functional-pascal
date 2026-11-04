# Distinct types

```pascal fragment
type UserId = distinct long;
type SampleName = distinct string;

const Id: UserId := new UserId(42L);
const Raw: long := cast<long>(Id);
```

A distinct declaration introduces a nominal type. Neither assignment nor a call
implicitly converts between it and its base type, or between different distinct
types with the same base. Transparent aliases retain the same nominal identity.

The base must resolve through aliases to a non-nullable primitive alias other
than object: a numeric type, bool, char, or string. It cannot be another domain,
an array, an arbitrary class, or a generic parameter. Generic distinct declarations
are not included.

`new UserId(Value)` constructs from the underlying type using ordinary argument
conversion rules. `cast<UserId>(Value)` wraps a base-typed value;
`cast<long>(Id)` unwraps it. The public read-only Value property exposes the same
underlying value. Converting between different domains requires an explicit
intermediate base value. Wrapping/unwrapping does not parse strings or implicitly
change primitive types. Boxing preserves the wrapper's nominal identity.

Same-domain equality compares underlying values. Ordered comparisons are
available for numeric and char bases; string domains compare ordinal contents.
Bool domains have equality only. There is no automatic arithmetic, implicit
printing, or method forwarding. Unwrap before using an API that requires the base.

A value-primitive domain emits an immutable CLR value-type wrapper with private
payload, read-only Value, typed equality, and explicit conversions. Its default
represents the primitive's default; no additional predicate is imposed.

A distinct string emits a sealed immutable reference wrapper. Its value constructor
rejects null with ArgumentNullException; there is no parameterless constructor.
This avoids a default struct containing null while promising a non-null string.
`SampleName?` is nullable-reference annotation; `UserId?` uses Nullable<UserId>.
Nullable value-domain casts lift their conversions. Nullable string payloads and
domain receivers require normal validation before wrapping, unwrapping, or ordered
comparison; equality still permits null reference operands.

## Example

```pascal
program Identities;

uses System.Console as Console;

type UserId = distinct long;
type SampleName = distinct string;

begin
  const Id: UserId := new UserId(42L);
  const Name: SampleName := new SampleName('Alpha');
  const Raw: long := cast<long>(Id);
  Console.WriteLine(Raw); // 42
  Console.WriteLine(cast<string>(Name)); // Alpha

  const Missing: UserId? := null;
  Console.WriteLine(Missing.HasValue); // false
end;
end program;
```

Passing Id to a long parameter requires explicit unwrapping. A separately declared
distinct long type is not assignment-compatible with UserId. Nullable UserId is a
CLR Nullable<UserId>, not an Option supplied by the language.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
