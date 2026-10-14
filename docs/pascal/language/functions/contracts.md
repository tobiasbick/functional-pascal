# Preconditions and postconditions

A named function or procedure can declare Boolean entry conditions after its
heading and before nested declarations or `begin`:

```pascal
function Divide(A: integer; B: integer): integer;
requires B <> 0;
begin
  return A div B;
end function;
```

Formal syntax: [grammar.ebnf](../../../specs/grammar.ebnf) (`requires_clause`, `ensures_clause`,
`function_decl`, `procedure_decl`).

Each `requires Expression;` must have type `boolean`. Repeated clauses run in
source order with short-circuit `and` behavior: the first false clause stops
the call. The checks run before local initialization and every body statement,
on every invocation, in debug and release builds. They also apply to nested
routines, generic routines, instance and static record routines, imported
routines, and calls through callable values or bound methods.

## Postconditions

Write `ensures Expression;` after any `requires` clauses and before nested
declarations or `begin`. Each expression must have type `boolean`; repeated
clauses run in source order and stop at the first false clause. Checks run in
every build mode, including release builds.

```pascal
function Increment(Value: integer): integer;
ensures (Incremented): Incremented = Value + 1;
begin
  return Value + 1;
end function;

procedure Set(var Value: integer);
ensures Value = 2;
begin
  Value := 2;
end procedure;
```

A function clause can name its complete return value with `(Name):` immediately
after `ensures`. The name has the declared return type and is visible only in
that clause. Each clause introduces its own name independently; it is unavailable
in the body and other clauses. Procedures have no return value to name. A leading
parenthesized identifier followed by `:` denotes this binding. The colon is
required: `ensures (Returned): Returned > 0;`. Without that complete prefix,
the clause is a normal expression: `ensures (Flag);` and
`ensures (Value > 0);` are ordinary Boolean conditions. There is no automatic
return-value name.

Postconditions run after the return expression is evaluated once, before the
value leaves the routine. They apply to early and final function returns,
explicit procedure returns, and normal procedure completion. A `try` error exit
also checks them: the named value is the complete `Error(...)` or `None`, rather
than an unwrapped success payload. For example:

```pascal
function Read(Input: result of (integer, string)): result of (integer, string);
ensures (Returned): case Returned of
  when Ok(const Value): Value >= 0;
  when Error(const Message): Message.Length() > 0;
end case;
begin
  const Value: integer := try Input;
  return Ok(Value);
end function;
```

Clauses read current parameter values, including completed writes through `var`.
There are no entry snapshots or `old` values. Writes are preserved if a clause
panics, under the existing [reference-write rules](var-parameters.md#evaluation-and-writes).
Panics and other runtime failures in the body are not returns and do not run
postconditions. A function still needs a return on every normal path; a clause
cannot provide a missing result, even if its condition is always false.

## Expression scope

Clauses may read the routine's parameters, including `Self` and its own
[`var` parameters](var-parameters.md), and visible initialized immutable values.
These include static and computed [`const` bindings](../basics/constants.md)
and immutable enclosing parameters or captures. A postcondition may also read
its own named return value. Body-local declarations are
outside the clause's scope. Mutable globals and mutable captures cannot be read.

Allowed forms are literals, operators, field and index reads, explicit type
conversions, typed construction, record updates, nonbinding `is` tests, and
`if`/`case` expressions.
Membership with `in` includes existing collection and subrange membership.
Every subexpression has the same restrictions, including arguments, guards,
indexes, and omitted record-field defaults. Defaults retain the binding scope
of their record declaration.

The following existing native receiver operations are allowed:

| Receiver | Operations |
|----------|------------|
| `string` | `Length`, `IsEmpty` |
| `array of T` | `Length`, `IsEmpty` |
| `dict of K to V` | `Length`, `IsEmpty` |

For example, `requires Text.Length() > 0;` checks a string parameter. Resolution
must select the actual native operation; a user function or record method named
`Length` remains an ordinary call and is rejected. Other native operations,
ordinary functions and methods, calls through callable values, anonymous
routines, `try`, `go`, and reference arguments are unavailable in clauses
([FP3037](../../tools/diagnostics.md#semantic-analysis)). Existing restrictions
on [`is` tests](../pattern-matching/is-test.md) still apply.

Clauses may read an already initialized callable constant or parameter as a
value, but may not form a routine value or a bound method inside the clause.
This keeps capture creation outside contract checks.

## Violations

A false clause causes a [panic](../error-handling/panic.md), with runtime
diagnostic FP5010. The message identifies the qualified routine, the original
failed clause text, and its current parameter names and values, including aggregate
values. A failed function postcondition also includes the complete returned
value, whether the clause names it or not. Argument expressions have already been evaluated under the normal
call rules; diagnostics read the resulting parameter values without evaluating
the arguments again. The location points to the failed clause, including when
the routine was imported from a compiled unit.

A violation is a programming error and does not become a `Result` error or an
`Option` value. Clauses do not supply a missing return and do not capture entry
snapshots or provide an `old` operation.

Both `requires` and `ensures` are reserved words.
