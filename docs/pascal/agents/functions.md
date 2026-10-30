# Functions, parameters and captures

Read this when declaring routines, passing callbacks, or changing caller state.

## Declarations and calls

Declare `function Name(Parameters): ResultType;` or `procedure Name(Parameters);`.
Functions explicitly return a value on every normal path. Procedures can exit
with `return;`. Parameter types and result types are explicit. Every parameter
has its own name/type; separate parameters with semicolons. Overloading and
default parameter values are unavailable.

Calls use commas and are either fully positional or fully named:
`Add(Left := 1, Right := 2)`. Arguments evaluate in written order even when labels
reorder parameters. A callable value uses the labels of its visible static type;
labels do not affect positional type compatibility. Variadic calls such as
`Console.WriteLn` are positional.

Named routine signatures are visible throughout their lexical scope, including
mutually recursive siblings. Values initialize in source order. Call, capture
or construct only when direct and transitive value dependencies are initialized;
unsafe or unprovable uses produce FP3041. Nested named routines appear before
the enclosing routine's `begin`.

Consume function results or explicitly use `discard Call(...);`. Discard accepts
every value type. It never retrieves a task result, cancels work or changes task
ownership. Eligible tail calls may reuse frames; contracts and task-scope exit
work can prevent reuse, and the debugger retains frames for inspection.

## Caller mutation

Ordinary value parameters are read-only. Copy one to a local `var` for local
changes. A `var` parameter aliases caller storage and requires the same type and
an explicit call marker: `Increase(var Count)` or `Increase(Value := var Count)`.
The designator is evaluated once. Writes are immediate and remain after an
early return or runtime failure.

Writable variables, their record fields, and array elements can be passed this
way. Constants, read-only parameters, dictionary entries, string scalars and
computed values cannot. Two `var` arguments in one call cannot share a storage
root, even through different fields or indices. `go` cannot pass `var` arguments.
Native InPlace receivers use their implicit mutable receiver without writing a
`var` marker. User record `Self` remains read-only.

## Callable values and closures

Callable types are `function(Value: T): R` and `procedure(Value: T)`.
Pass a named routine without `@`. Anonymous routines use a complete
`function(...)...begin...end function` or `procedure(...)...begin...end procedure`
expression. Bind a returned callable before invoking it; `MakeAdder(3)(5)` is
unsupported.

Native routines and factories are callable values too (`Math.Abs`, `array.Fill`);
a receiving signature specializes them. `Value.Operation` captures its receiver
once by value. `Some`, `Ok` and `Error` also supply payload-constructor functions;
`None` remains a value. See [native operation values](../language/functions/first-class.md#native-operation-values).

Ordinary captured parameters and `const` values are snapshots; a captured local
`var` is a shared mutable cell that can outlive its declaring call. Such callables
stay in their owning task. Anonymous closures cannot capture `var` parameters.
A directly called named nested helper may borrow one, but cannot become a
retained callable or task while it uses the reference. Globals remain globals.

## Contracts

After a named routine header, write zero or more `requires` clauses followed by
`ensures` clauses, before nested declarations and `begin`. Clauses are read-only
Boolean expressions over parameters and permitted immutable outer values;
body locals and mutable outer state are unavailable. Ordinary function calls
are restricted; consult [contracts](../language/functions/contracts.md) for
permitted native queries, construction and `Error.Message()`.

`ensures (Answer): Expression;` binds the whole return value in that clause;
the colon is required. Without it, `(Flag)` is an ordinary grouped expression.
Postconditions observe completed `var` writes and also run on `try` propagation.
Runtime failures in the body do not run postconditions. There is no old-value
snapshot. Contracts execute in all build modes by default; `--no-contracts`
omits their execution across dependencies while keeping parsing/type checking.

```pascal id=caller-state-and-capture run
program AgentFunctions;

uses Std.Test as Test;

procedure Increase(var Value: integer);
ensures Value > 0;
begin
  Value := Value + 1;
end procedure;

function MakeAdder(Base: integer): function(Value: integer): integer;
function Add(Value: integer): integer;
begin
  return Base + Value;
end function;
begin
  return Add;
end function;

begin
  var Count := 0;
  Increase(Value := var Count);
  const AddThree: function(Input: integer): integer := MakeAdder(3);
  Test.AssertEquals(8, AddThree(Input := 5));
  Test.AssertEquals(1, Count);
end.
```

Further rules: [parameters](../language/functions/parameters.md),
[`var` lifetimes](../language/functions/var-parameters.md),
[closures](../language/functions/closures.md), and
[generic routines](../language/functions/generic-routines.md).
