# Discarding values

`discard Expression;` evaluates an expression exactly once and drops its value.
`discard` is a reserved, case-insensitive keyword. Every value type is accepted,
including task handles, aliases, aggregates, channels, unconstrained generic
values, and callables with captures. No task-free proof or generic constraint
is required for discard.

```pascal
discard 42;
discard Some('unused');
discard Error('deliberately ignored');
```

A discarded Result or Option is not unwrapped. Expression side effects and
runtime failures still occur. Discarding a function or procedure value does
not invoke it. A procedure call produces no value and is rejected as an
operand (FP3020); call the procedure directly.

Every function result must be consumed: assign it, return it, pass it to another
call, inspect it with `case`, or explicitly discard it. An unused result is
FP3022, including the final function call of a postfix chain and a standalone
go expression. The repair is valid independently of the returned value's type
or captures.

```pascal
const Value: integer := Compute();
discard Compute();
discard Factory.Create().Compute();
```

Use `case` when the branches of a Result matter, or `try` to propagate an
error while consuming the success value. Explicit discard deliberately drops
both branches.

## Task handles and aggregates

Discarding a handle does not cancel, detach, join, or consume its task result.
The lexical scope or explicit TaskGroup retains the child independently of
user handles. Panic/runtime failures remain monitored by that owner; returned
Error values remain ordinary Result data. Retrieve required results with Wait
before the owner releases them.

```pascal
scope
  const Job := go Worker();
  discard Job;
  discard [Job];
  discard go Worker();
end scope;
```

The scope still joins both children. Discard does not relax ordinary expression
typing, [scope-handle escape rules](../concurrency/task-scopes.md), initialization,
or cross-task restrictions on foreign mutable state and task-bound captures.
Discarding a callable is ordinary disposal; transferring or invoking it across
a task boundary remains separately checked.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`discard_stmt`).

## See also

- [Runnable discard example](../../../../examples/pascal/functions/discard_values.fpas)
- [Capturing closures](closures.md)
- [Task handles](../concurrency/task-handles.md)
- [Channels](../types/channels.md)
- [Diagnostics](../../tools/diagnostics.md)
