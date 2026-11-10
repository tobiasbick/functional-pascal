# Bindings and assignment

Declare one explicitly typed, initialized binding at a time. Const prevents
rebinding; var permits assignment to writable storage.

```pascal fragment context=body
const Name: string := 'Alpha';
var Remaining: int := 10;
const Limit: int := 100;
Remaining := Remaining - 1;
```

Every `const` or `var` declaration introduces one name and requires both an
explicit type and an initializer, at every scope. The syntax is
`const Name: T := Expression;` or `var Name: T := Expression;`. Omitting `: T` is
an error even when the initializer's type is obvious. The initializer must be
implicitly convertible to the declared type; the binding retains that type.

The same explicit-type rule applies to loop variables, using resources, and
pattern bindings. Parameters, function results, and record fields also require
types. A named ensures result uses `ensures (Returned: T): Condition;` and also
requires its annotation. A CLR type pattern states its binding's type before `as`; a wildcard
introduces no binding. Compiler-provided `self` and `args` have the fixed types
defined by their enclosing record and program.

The declared type supplies context to empty array literals, `null`, and generic
variant constructors. Expression typing, overload resolution, and generic argument
inference still operate; they never remove a declaration's type requirement.

`const` means an immutable **binding**, not necessarily a compile-time constant.
Its initializer may execute at runtime. A `const` reference can still refer to a
mutable object; calling `Items.Add(1)` is legal on a `const` list. `var` allows
rebinding. Parameters are read-only bindings unless declared `ref` or `out`.

Assignment evaluates the receiver and indexes once, then the right side, in
source order. It can target a local `var`, a writable CLR field/property/indexer,
a writable array element, or a `ref`/`out` parameter. Properties may have effects.
It can also target an accessible top-level var, emitted as a CLR static field.
It cannot target a language record's read-only field or a domain's read-only payload.

There are no implicit declarations or implicit conversions from values to `bool`.
All statements end with `;`, including the last statement before `else` or a named
`end`. An empty statement list is allowed. A nested `begin ... end;` creates scope.
Programs close with `end program;` and units with `end unit;`; neither has a
terminal period. The program's entry block first closes with its own `end;`.
See [program structure](../../program-structure/README.md).
Local resource bindings use `using const Name: T := Expression;` (or
`await using const` for async disposal), without a separate block closer.
Their cleanup runs on leaving the enclosing lexical scope; see
[errors and resources](../error-handling/README.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
