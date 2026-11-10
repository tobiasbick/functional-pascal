# Absence and domain outcomes

Use CLR nullability (`T?`) for a value that may be absent. Use the API's existing
Try-pattern where provided:

```pascal fragment context=body
var Count: int := 0;
if int.TryParse('42', out Count) then
  Console.WriteLine(Count);
else
  Console.WriteLine('Expected an Int32 value');
end if;
```

Application variants describe normal domain alternatives, such as a move that
succeeds or is blocked. They are ordinary return values, not another exception
mechanism. Each application chooses its meaningful cases and handles them
explicitly; see [variants](../types/variants.md).

There is no built-in Option/Result convention or postfix failure-propagation
operator. A variant returned by a .NET or FPAS call is never automatically unwrapped.
An exception thrown while computing any return value still propagates normally.

## Domain and library conventions

Use `T?` for nullable values and CLR exceptions for failures. A domain operation
can return its own variant when several normal outcomes have different payloads;
see the [gameplay example](../types/variants.md#example).
Such a variant is ordinary data: it neither unwinds the stack nor causes an implicit
return when one case is encountered. Inspect it with case matching.

Option and Result are not predefined types, implicit imports, or required
Fpas.Core APIs. A future library or application may define them as ordinary
variants, with no compiler-specific failure behavior. The language has no postfix
failure-propagation `?`; the `?` type suffix denotes CLR nullability.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
