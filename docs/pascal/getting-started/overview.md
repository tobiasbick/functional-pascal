# Overview

Functional Pascal is a modern, function-first programming language built on Pascal's readable syntax. It runs on a managed virtual machine — no pointers, no manual memory management, no unsafe operations.

## Design philosophy

- **Function first** — Functions are the primary building block. No classical classes.
- **Explicit binding mutability** — `const` bindings are immutable; `var` bindings permit reassignment.
- **Explicit types** — Parameters, routine results, and record fields declare their types. Local bindings allow [type inference](../language/basics/local-variables.md#type-inference) whenever the initializer's type is fully determined.
- **Safe by design** — The VM manages memory. No pointers, no manual allocation.
- **Familiar syntax** — Pascal's `begin`, `end`, `:=`, `downto` and other well-known keywords.
- **Case-insensitive** — Keywords and identifiers are case-insensitive, following Pascal tradition.

## Example

```pascal id=example-1
program Greeting;

uses Std.Console as Console;

begin
  Console.WriteLn('Hello, Functional Pascal');
end.
```

## See also

- [Hello world](hello-world.md)
- [First program](first-program.md)
- [Keywords](keywords.md)
