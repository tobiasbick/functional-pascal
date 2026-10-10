# AP09.1: Named arguments for routines and methods

Package: [AP09: Named arguments](README.md)

Status: complete.

## Result

Declared routines and record methods accept positional or fully named calls.
Names match case-insensitively and use declared parameter names. Parameters
have no defaults; unknown, duplicate and missing names are FP3025, while mixed
lists are FP2016. Generic inference uses mapped arguments. Evaluation remains
once per argument in written order, followed by parameter-order passing.

Fixed-signature native type operations also accept names through
[AP06.3](../ap06-dot-call-targets/03-fixed-dot-resolution.md). Their implicit
receiver is unnamed and evaluated first. Function values and callable record
fields/properties are positional only, as are variadic routines and native
`Format` (FP3026). Ordinary polymorphic Std routines retain their positional-only
restrictions where no fixed public signature exists.

Signature help, navigation and rename resolve parameter labels to the callee.
Debugger evaluation supports fully named calls to declared routines, methods,
and enum constructors; see the [debugger handbook](../../../pascal/tools/debugger.md).

## Regression coverage

Parser, sema, compiler, CLI and editor tests cover reordered traces, same-typed
roles, imports, generics, methods, invalid mappings and formatting.
See [parameters](../../../pascal/language/functions/parameters.md).
