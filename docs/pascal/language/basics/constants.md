# Constants

Each constant declaration starts with its own `const` keyword and uses `:=`
for its compile-time-known value. Declaration groups are rejected with
[FP2015](../../tools/diagnostics.md#parser). Constants are declared at program
or unit level; routine bodies allow local variable bindings.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_declaration`, `const_def`).

```pascal
const Pi: real := 3.14159265;
const MaxSize: integer := 1024;
const Greeting: string := 'Hello';
```

## See also

- [Variables](variables.md)
