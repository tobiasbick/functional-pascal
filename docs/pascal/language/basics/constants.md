# Constants

Each constant declaration starts with its own `const` keyword and uses `:=`
for its initial value. A `const` binding cannot be reassigned and may be
initialized by a runtime expression, including a function call. Constants are
allowed at program and unit level and inline in statement lists, including
routine bodies and nested control-flow scopes. Declaration groups are rejected
with [FP2015](../../tools/diagnostics.md#parser).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_declaration`, `const_def`, `const_stmt`).

```pascal
const Pi: real := 3.14159265;
const MaxSize: integer := 1024;
const Greeting: string := 'Hello';
```

## Initialization

A local initializer runs once whenever execution reaches its declaration, in
statement order. Inside a loop it runs on each iteration that reaches the
declaration; an untaken branch, empty loop, or earlier return does not run it.
Put the declaration before a loop to compute its value once for that execution
of the surrounding statement list. An initializer failure occurs at that point.

Program and unit initializers retain declaration order; dependencies initialize
before consuming units and the program body. See [Units](../../program-structure/units.md).
Closures copy captured local constants. Immutability applies to the binding;
shared resource handles retain their own operations.

## Compile-time constants

Literal values, references to compile-time constants or enum members, and
existing operator, array, dictionary, record, `Some`, `None`, `Ok`, and `Error`
forms remain compile-time known when all their inputs are known. Omitted record
field defaults also participate in this classification.

Every function or method call is computed at runtime, including standard-library,
intrinsic, and native type-operation calls. A call stays computed even if it
returns a literal. References to computed constants and expressions derived from
them also stay computed. Optimization does not change this classification.

Scalar `case` value labels and both range endpoints require compile-time
constants. A diagnostic names the offending call or binding. Use a
[guard](../pattern-matching/guards.md) for a dynamic condition:

```pascal
const Expected: integer := ReadExpected();
case Actual of
  when Value if Value = Expected:
    HandleMatch();
  else
    HandleOther();
end case;
```

Exported constants retain this distinction across compiled units. Static scalar
values can be embedded in consumers; computed constants and aggregate constants
are read from immutable unit globals after initialization.

## See also

- [Variables](variables.md)
