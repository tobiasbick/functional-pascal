# Supported signatures and limits

The initial compiler must reject unsupported metadata with a specific diagnostic.
In particular, it must not pretend that ref structs/Span<T>, pointers, function
pointers, ref returns, unmanaged calling conventions, expression trees, arbitrary
custom awaiters, init-only object initializers, or dynamic binding are ordinary
supported calls. Consuming ordinary managed APIs is the initial interoperability
target; complete C# language and ABI feature coverage is not assumed.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
