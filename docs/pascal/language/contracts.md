# Preconditions and postconditions

Contracts express executable expectations on a routine. `requires` checks its
inputs before the body runs; `ensures` checks its state/result on successful exit.
They are normal runtime checks in the emitted CLR method, in every build mode.
They are not debug-only assertions, static proofs, or a separate failure mechanism.

## Syntax

```pascal
function AddPositive(Left: int; Right: int): int;
requires Left > 0;
requires Right > 0;
ensures (Returned: int): Returned > Left;
begin
  return Left + Right;
end function;
```

The order is routine header, generic `where` constraints, zero or more requires
clauses, zero or more ensures clauses, then `begin`. A requires clause after an
ensures clause is invalid. Each clause ends with a semicolon.

- `requires Condition;` requires a `bool` expression.
- `ensures Condition;` checks a `bool` expression without naming a result.
- `ensures (Returned: T): Condition;` names the successful result inside that
  clause. The explicit annotation is mandatory and must match the result type
  after resolving aliases. It does not convert or filter the result.

The result name is a read-only binding scoped to its own condition. It cannot
shadow a parameter or `self`. It is unavailable in requires clauses, the body,
and other ensures clauses. Different clauses may each declare their own typed
result binding. Parentheses around an ordinary Boolean condition do not declare
a result: `ensures (Value >= 0);` is a result-free condition.

Contracts apply to named free/nested routines and record methods, including
methods implementing CLR interfaces. Procedures may use result-free ensures:

```pascal
procedure ClampToZero(ref Value: int);
ensures Value >= 0;
begin
  if Value < 0 then
    Value := 0;
  end if;
end procedure;
```

Block-bodied anonymous functions/procedures may put contracts between their
signature and `begin`, without a declaration-header semicolon:

```pascal
const Double: function(int): int := function(Value: int): int
  requires Value >= 0;
  requires Value <= int.MaxValue div 2;
  ensures (Returned: int): Returned = Value * 2;
begin
  return Value * 2;
end function;
```

Expression-bodied lambdas cannot carry contracts; use a block body. Contracts
are not part of a callable type, generic constraint, or CLR method signature.
They are not declared on program entry bodies, records as a whole, or imported
methods. The baseline has no inherited contract or object-invariant syntax.

## Execution and cleanup

Evaluate arguments normally, then enter the callee. Run requires clauses once,
in written order, before executing the routine body. The first false condition
throws `System.InvalidOperationException` and prevents body execution.

On a normal return:

1. Evaluate and convert the return expression once and retain the result.
2. Complete all finally/using cleanup required by leaving the body.
3. If cleanup succeeds, run ensures clauses once in written order.
4. Publish the retained result, or complete the procedure.

Falling through the end of a procedure follows the same exit path. Early returns
do not bypass postconditions. If the body, return-expression evaluation, or
cleanup throws, the exit is exceptional and ensures clauses do not run.

A false ensures condition throws `System.InvalidOperationException` instead of
publishing a successful result. Diagnostic messages identify the routine, clause
kind, and failed condition, with a source location when available. Do not evaluate
the condition or its operands again to produce the message.

An exception thrown while evaluating a condition propagates unchanged; it is not
wrapped as a false-condition failure. No automatic rollback of previous effects
is performed. In particular, a failed postcondition does not undo body mutations.

Generated entry/exit checks are outside the body's own try/catch handlers; callers
can catch their CLR exceptions. Cleanup has already completed when ensures runs,
so a postcondition failure cannot skip or repeat that cleanup. The callee owns
the checks, making calls through delegates, interfaces, or another .NET language
follow the same rules. They must not depend on call-site rewriting or optional
.NET Code Contracts tooling.

## Condition scope and allowed expressions

All conditions are read-only `bool` expressions. Requires sees initialized
parameters, `self` in a record method, and accessible compile-time constants.
Nested/anonymous routines may also read initialized outer bindings under the
same capture rules as their body; contract-only uses also establish captures.
An out parameter is uninitialized at entry and cannot be read by requires.
Ensures sees the final parameter state, initialized out parameters, `self`,
constants, captures, and its optional result binding. Locals declared inside
that routine's own body are not in scope.
Global vars are not in contract scope. Static record methods have no self;
private fields may be read only where record access rules permit them.

The initial language restricts contract expressions to a checkable subset:

- Literals, compile-time constants, and bindings available in the condition scope.
- Built-in scalar operators/conversions/comparisons, null/type tests, Boolean
  short-circuiting, coalescing, and conditional expressions.
- Reads of language-record fields and language-union payloads selected by a
  pattern, array elements, string indexes, vector/string Length, and the standard
  Nullable<T> HasValue/Value members. Ordinary null/bounds checks still apply.
- Compiler-defined domain unwrapping, Value reads, and scalar-domain comparisons
  whose underlying operations belong to this subset. Wrapping/constructing a
  domain is not permitted in a condition.

Method/delegate calls, constructors, lambdas, record updates, arbitrary CLR
property/indexer getters, user-defined operators/conversions, and generated
record/union equality are not allowed. Their execution may have effects through
foreign code. Assignment, await, subscriptions, ref/out calls, and control-transfer
statements are also forbidden. This restriction requires no global purity
inference or handwritten catalog of pure library methods.

Reference/array identity tests and primitive equality remain available; the
restriction on generated aggregate equality prevents hidden user Equals calls
inside EqualityComparer<T>.Default. An identity/null comparison is allowed only
when ordinary binding selects a built-in operation, not an overloaded equality
operator; contract syntax does not change operator resolution. A built-in
operation may still throw, for example checked arithmetic or an invalid array index.

There is no `old` operator, entry snapshot, deep-copy rule, or lock acquired for
a contract. Reference state read by ensures is its state after the body/cleanup,
not an implicit snapshot of its entry state. Shared data may change concurrently;
contracts do not establish synchronization or a data-race guarantee.

## Async routines

```pascal
async function DoubleAfterDelay(Value: int): Tasks.Task<int>;
requires Value >= 0;
requires Value <= int.MaxValue div 2;
ensures (Returned: int): Returned = Value * 2;
begin
  await Tasks.Task.Delay(1);
  return Value * 2;
end function;
```

Inside an async Task<T> function, a typed ensures result is **T**, not Task<T>.
Run requires before the user body in the async state machine. Run ensures after
successful body/async cleanup and before completing the Task successfully.
Do not rerun contracts at each await or when a consumer awaits the same Task.

A contract violation faults the returned Task according to ordinary async
exception rules, including a requires violation before the first await. It does
not introduce a synchronous guard wrapper. A faulted or canceled operation skips
ensures. A non-generic async Task function supports only result-free ensures.
Contract expressions themselves cannot await or initiate work.

In a synchronous function that merely returns Task<T>, ensures instead sees the
actual Task<T> object at synchronous return. It does not inspect later completion
or attach a continuation. These cases follow the declared routine's `async`
modifier, not just the presence of Task in its result type.

## Implementation checks

A future compiler must exercise true/false conditions, ordered clauses, typed
result bindings, multiple/early returns, procedure out/ref state, generic routines,
block lambdas, and interface/delegate/cross-language calls. Cover normal cleanup,
cleanup failure, body failure, condition-evaluation exceptions, and async
success/fault/cancellation. Verify conditions execute once in debug and release.
Reject missing/mismatched result annotations and disallowed expressions.

The [synchronous implementation](../../implementation/contracts-and-parameters.md)
covers free routines, record methods, generic routines, and block lambdas.
Named nested routines and async contracts are implemented. Authored interfaces
are a [deferred extension](../../future/language-gaps.md#deferred-declare-fpas-interfaces),
outside the original language baseline.
