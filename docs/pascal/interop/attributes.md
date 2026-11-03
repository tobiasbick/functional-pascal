# CLR attributes

Source CLR attributes are [implemented](../../implementation/attributes.md),
including usage checks, constant values, property mapping, and supported CLR flags.

```pascal fragment
type Player = record
  [Serialization.JsonPropertyNameAttribute('player_name')]
  Name: string;
end record;
```

Attribute lists precede a type, routine, or record field declaration. Specify the
exact attribute type name, including its Attribute suffix. Constructor arguments
are positional. Following named arguments set public writable attribute properties
or fields, using `Name := ConstantExpression`; they do not name constructor
parameters. Values must be legal CLR attribute constants, typeof expressions,
or supported arrays of those values.

A field annotation on a language record applies to its emitted property.
Routine attributes apply to its emitted method. AttributeUsage and constructor
binding are checked. Assembly/return/parameter/backing-field attribute targets are
not part of the initial source syntax. Compiler-generated nullability and parameter
metadata are emitted without requiring user-written attribute syntax.

Use ordinary CLR custom-attribute constants, closed typeof expressions, and array
literals. Decimal/native/nullable value wrappers and runtime expressions are not
attribute constants. Aliases have no emitted type target; globals and explicit
assembly/parameter/return/backing-field targets are outside this syntax.
Serializable and supported managed-IL MethodImpl options map to CLR flags.
Unsupported layout/native-interoperability pseudo-attributes are diagnosed rather
than written as ineffective custom blobs. Compiler-owned metadata and declared
nullable record-property types cannot be contradicted by source annotations.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
