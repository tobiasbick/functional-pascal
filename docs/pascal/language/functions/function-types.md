# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal id=example-1 fragment context=declarations
type IntBinaryOp = function(A: integer; B: integer): integer;

type StringAction = procedure(S: string);

type Step = procedure(var Value: integer);
```

A parameter declared with `var` changes the caller's variable; see
[`var` parameters](var-parameters.md). Parameter modes are part of the type:
`procedure(Value: integer)` and `procedure(var Value: integer)` are different
types, and neither routine can be assigned to the other type.

Parameter names are required and unique without regard to case, including
names in nested callable types. They are the labels for named calls through
that static type: a value of `function(A: integer; B: integer): integer`
accepts `F(B := 2, A := 10)` or `F(10, 2)`. Each call uses one form throughout
and supplies every parameter once.

Names do not participate in assignment or callback compatibility. A routine
with parameters `Left` and `Right` may implement this type; calls through the
receiving value still use `A` and `B`. No runtime lookup of implementation
parameter names occurs. See [named arguments](parameters.md#named-arguments).

## Payload constructors as functions

A payload enum or native-error constructor is an ordinary noncapturing function
value. Its fields supply readonly parameters and its result is the declaring
family. Generic constructor arguments must be inferable from the receiving
callable signature, including parameters and result. The normal ambiguity
diagnostic rejects missing evidence.

```pascal id=example-2 run
program Constructors;
uses Std.Test as Test;
type Number = enum
  Item(Value: integer);
end enum;
begin
  const Make: function(N: integer): Number := Number.Item;
  Test.AssertTrue(Make(N := 7) is Number.Item(7));
end.
```

The receiving callable's labels govern calls, just as for named routines.
Multiple payload fields require matching arity; there is no currying or automatic
capture. Fieldless variants remain values and never accept `()`.

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
