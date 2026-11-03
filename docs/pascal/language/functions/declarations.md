# Declarations and calls

Declare typed functions/procedures and consume their results explicitly.

```pascal fragment
public function Add(Left: int; Right: int): int;
begin
  return Left + Right;
end function;

procedure Show(Text: string);
begin
  Console.WriteLine(Text);
end procedure;
```

Parameters are separated by `;`, call arguments by `,`. Every routine has
parentheses, even without parameters. A function declares a result type and
returns a value on every normal path. A procedure returns no value and may use
`return;`. An expression-bodied declared routine is not part of the baseline.

Optional `requires` and `ensures` clauses appear after any `where` constraints
and before `begin`. They check routine entry and successful exit in all build
modes; see [contracts](contracts.md) for typed result bindings, expression limits,
cleanup ordering, and async behavior.

Contiguous standalone `//` comments immediately before a declaration document it
as Markdown. Structured Parameters, Type parameters, Returns, and Remarks sections
map to the optional .NET XML sidecar; see [comments and API documentation](../basics/comments.md).
Documentation does not alter a routine's signature or its executable contracts.

`Add(1, 2)` and `Add(Right := 2, Left := 1)` are valid. Positional arguments may
precede named arguments, but not follow them. Arguments evaluate once in written
order, even when names reorder their receiving parameters. Labels match the
selected declaration's parameter names exactly. Duplicate arguments are errors.

Parameters are immutable bindings by default, not deeply immutable objects.
Default parameter values use `Name: T := ConstantExpression`; required parameters
must precede optional parameters. Defaults must be CLR-compatible compile-time
constants and are inserted at the call site. `const` declarations with runtime
initializers do not qualify as default expressions.
Decimal defaults use ordinary CLR DecimalConstantAttribute metadata. Native
integer defaults must fit Int32/UInt32, the CLR constant metadata representation
used for these types independently of the execution architecture.

Consume a function result or write `discard Expression;`. A bare value-producing
call statement is an error. A procedure/CLR void call is a statement. Discarding
a task does not wait, cancel, supervise, or detach it from a language owner:
there is no such owner. See [concurrency](../concurrency/README.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
