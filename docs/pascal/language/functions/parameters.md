# Managed parameter references

Use managed modes when a routine must read or update caller storage directly.

```pascal fragment context=declarations
procedure Increment(ref Value: int);
begin
  Value := Value + 1;
end procedure;
```

- `ref Value: T` aliases writable caller storage; call with `ref Storage`.
- `out Value: T` aliases storage the callee must assign before normal return;
  call with `out Storage`. The callee cannot read it before assignment.
- `in Value: T` is a read-only managed reference; call with `in Storage`.
- An unmodified parameter passes by value.

Calls require addressable storage of the exact element type, with no numeric
conversion. Locals passed to `out` must still be declared with an initializer.
An ordinary CLR property is not addressable storage. Writable fields and array
elements may be passed as appropriate. Source `const` bindings are not writable
`ref`/`out` targets.

Byref parameters cannot be captured, returned by reference, stored in objects, or
used as async parameters. Async calls with ordinary byref arguments are allowed
only if the invoked method's CLR signature itself supports that synchronous call;
the caller must not keep a managed reference across suspension. No pointer,
`ref struct`, ref return, or byref-local syntax is provided.

`params Values: T[]` may be the final by-value parameter. A call may supply an
existing array or expanded element arguments. Expanded arguments allocate one
array. The same rules apply to imported CLR `ParamArray` parameters.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
