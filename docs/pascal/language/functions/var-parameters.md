# `var` parameters

A `var` parameter lets a routine change a variable of its caller. Parameters
are read-only unless they are declared with `var`. Every explicitly written argument for a `var`
parameter is marked with `var` at the call site, so a call that changes caller
state is visible where it is written:

```pascal
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

procedure Swap(var A: integer; var B: integer);
begin
  const Temp: integer := A;
  A := B;
  B := Temp;
end procedure;

begin
  var Counter: integer := 0;
  Increase(var Counter);   // Counter is 1
end.
```

Named calls put the marker after the parameter label:

```pascal
Increase(Value := var Counter);
Swap(B := var Right, A := var Left);
```

The call is fully positional or fully named. Names select parameters; arguments,
including the roots and indices of `var` arguments, are evaluated once in written
left-to-right order. The same storage, type, aliasing, and lifetime rules apply
to both forms. See [Named arguments](parameters.md#named-arguments).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf)
(`param_group`, `named_arg`, `var_arg`).

## Arguments

A `var` argument names storage the caller can change:

- a `var` binding (local, program, or unit variable, including a public `var`
  of an imported unit);
- a field of such a record, `var P.X`;
- an element of such an array, `var Items[I]`;
- a `var` parameter of the enclosing routine, which forwards the caller's
  variable: `Increase(var Value)`.

Constants, `const` bindings, read-only parameters, loop variables, dictionary
entries, string characters, and computed values cannot be passed as
`var` (FP3028). Copy such a value into a local `var`, pass that, and assign
the result back.

The argument has exactly the parameter type, because the routine writes values
of that type back (FP3006). A `var` parameter requires the `var` marker, and the
marker is rejected for a read-only parameter (FP3027).

Two `var` arguments of one call must not refer to the same variable, including
different fields or elements of it (FP3029):

```pascal
Swap(var A[I], var A[J]);   // FP3029: both arguments refer to `A`
SwapAt(var A, I, J);        // pass the array once instead
```

The check covers the arguments of one call. A routine that also changes the
same variable through another name, such as a unit variable it writes directly,
sees the change through both names.

## Evaluation and writes

The variable, field, or element of each `var` argument is determined once,
when its argument is evaluated: its root and indices are evaluated in written
order together with the other arguments. Inside the routine, every read of the
parameter reads the caller's current value, and every assignment updates the
caller's variable immediately.

Writes that completed before a runtime failure or an early `try` exit remain in
the caller's variable; nothing is rolled back:

```pascal
function Partial(var Value: integer): Result of integer, string;
begin
  Value := 1;
  return Error('stopped');
end function;
```

After `Partial(var Value)` returns its error, `Value` is `1`.

An element argument refers to its index fixed at the call. If the array becomes
shorter while the routine runs, using that parameter fails with an
array-index error (FP5003).

## Lifetime

A `var` parameter refers to the caller's variable only while the call runs:

- An anonymous closure cannot capture a `var` parameter (FP3030). Copy the value
  into a local first.
- A named nested routine can use an enclosing `var` parameter when it is called
  directly. It cannot be used as a routine value or started with `go` while it
  uses one (FP3030).
- `go` cannot pass a `var` argument (FP3030).

## Function types

Function and procedure types can declare `var` parameters. Parameter modes are
part of the type: a routine with a read-only parameter and one with a `var`
parameter cannot be substituted for each other. Calls through a function value
mark `var` arguments the same way:

```pascal
const Step: procedure(var Value: integer) := Increase;
Step(var Counter);
```

See [Function types](function-types.md).

## Restrictions

- The record receiver `Self` cannot be a `var` parameter.
- A receiver call such as `Counter.Increase()` cannot supply a `var` first
  parameter; write `Increase(var Counter)`.
- Array `Push` and `Pop` accept an implicit writable receiver:
  `Items.Push(Value)` and `Items.Pop()`. It has no receiver marker or additional
  parentheses and uses the same storage, aliasing, evaluation, and lifetime
  checks. Fields, array elements, and forwarded `var` parameters are accepted;
  `const` and computed receivers are rejected. Explicit ordinary-call arguments
  retain `var`, for example `Push(var Items, Value)`. See
  [Mutating arrays](../types/array/mutating.md).

## See also

- [Parameters](parameters.md)
- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [`go`](../concurrency/go.md)
