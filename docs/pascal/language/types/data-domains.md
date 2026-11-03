# Data domains

Distinct types and checked integral subranges create nominal CLR domains. Every value binding has an explicit type.

## CLR interoperability

Public domains emit named CLR types, not erased primitive signatures. Other
.NET languages use their constructors, Value, and conversions. Compiler metadata
identifies distinct bases and subrange bounds for cross-assembly FPAS consumers.

Generic constraints inspect the emitted wrapper type; being based on int does
not automatically implement all Int32 interfaces or generic-math traits. Do not
forward arbitrary base members or bypass bounds for generic instantiations.

See the complete examples for [distinct types](distinct-types.md#example) and
[subranges](subranges.md#example).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
