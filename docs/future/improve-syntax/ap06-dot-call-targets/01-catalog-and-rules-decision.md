# AP06.1: Catalog names and remaining operation rules

Package: [AP06: Fixed dot-call targets](README.md)

Status: complete.

## Result

The [catalog](catalog.md) defines 78 entries: 75 distinct preserved operations
and `IsEmpty` for strings, arrays and dictionaries. Static receiver type and
case-insensitive operation name select one signature. Strings use array naming
for equivalent operations, including `Slice(Start, Len)`.

Operations are available without imports. Instance operations use dot calls;
`string.Chr(N)` and `array.Fill(Value, Count)` are the only factory forms.
Fixed signatures accept positional or fully named explicit arguments; variadic
`Format` is positional. Receiver-first evaluation and writable Push/Pop safety
are specified together with collision and generic-inference rules.

The compiler and editor use `crates/fpas-sema/src/std_registry/native/`.
No language question remains for this package.
See [catalog coverage](inventory.md).
