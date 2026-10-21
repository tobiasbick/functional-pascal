# Capturing closures

Anonymous `function` / `procedure` expressions create callable values that own a
managed lexical environment. The value uses the existing function or procedure type
whose signature matches the closure.

Parameter and result annotations are mandatory. Anonymous functions close with
`end function`; anonymous procedures close with `end procedure`. Their body
statements require `;`, including the final statement before the named ending.
An empty routine body remains valid.

The expression has no terminating `;` of its own. An enclosing declaration,
assignment, or `return` statement supplies its terminator. As a call argument,
the named ending is followed by `,` or `)` without an intervening `;`:

```pascal id=example-1 fragment context=closure-apply
Apply(
  function(Value: integer): integer begin
    return Value * 2;
  end function
);
```

Here the `;` after `)` terminates the call statement.

```pascal id=example-2 fragment context=body
var Count: integer := 0;

const Increment: procedure() := procedure() begin
  Count := Count + 1;
end procedure;
const AddBase: function(Value: integer): integer := function(Value: integer): integer begin
  return Count + Value;
end function;
```

Closures may be stored in variables and records, passed as arguments, returned from
routines, and invoked through ordinary positional or fully named call syntax.
Calls use the receiving static signature's labels, which may differ from the
closure's own parameter names; see [function types](function-types.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`closure_expr`).

## Capture rules

Capture is lexical and automatic. A name is captured when the closure body refers to
a local, parameter, or enclosing capture that is not declared by the closure itself.

| Binding | Capture behavior |
| --- | --- |
| Value parameter | Capture its value when the closure is created. |
| `var` parameter | Rejected (FP3030): the closure could outlive the call. Copy the value into a local first. |
| `var` local | Capture one shared mutable cell. |
| Enclosing closure capture | Reuse the same value or mutable cell. |
| Unit or program variable | Resolve normally; not stored in the closure environment. |
| Local `const` binding | Copy the value when the closure is created. |
| Routine, static record routine, or program/unit constant | Resolve normally; not stored as capture data. |

All closures created by one activation and capturing the same mutable local observe
the same cell. The cell belongs to the creating logical task, even when that task
moves between worker threads. Synchronous callbacks share that ownership.

```pascal id=example-3 fragment context=declarations
function Counter(): function(): integer;
begin
  var Value: integer := 0;
  return function(): integer begin
    Value := Value + 1;
    return Value;
  end function;
end function;
```

There is no capture-list syntax. Local bindings use `const` or `var`;
read-only parameters are captured by value, and `var` parameters cannot be
captured by an anonymous closure.

## Named nested routines

A nested named routine that refers to enclosing locals becomes a capturing closure
when it is used as a first-class value (assigned, returned, or passed):

```pascal id=example-4 fragment context=declarations
function MakeAdder(Base: integer): function(Value: integer): integer;
function Add(Value: integer): integer;
begin
  return Base + Value;
end function;
begin
  return Add;
end function;
```

Non-escaping nested helpers that are only called by name while their parent frame is
active keep the existing nested-function behavior.

Named helpers are visible throughout their lexical scope, including mutually
recursive siblings. Captured bindings must be initialized when the callable
environment is created; calls must also have initialized direct and transitive
dependencies. An unsafe or unprovable use produces `FP3041`. A callable stored
in `const` or `var` retains ordinary source-order initialization. See
[declaration order](../types/declaration-order.md).

A routine's captures include the captures needed by the named sibling routines
it calls or uses as values. Each capture retains its original lexical binding,
even when the caller has a parameter or local with the same name. Shared mutable
cells, reference storage, and task restrictions propagate through the call chain.

A named helper that directly or indirectly uses an enclosing `var` parameter
can only be called by name while the enclosing call runs. Using it as a routine
value, starting it with `go`, or calling it from an anonymous closure is rejected
with FP3030. See [Reference-parameter lifetimes](var-parameters.md#lifetime).

## Lifetime and equality

Creating or copying a closure copies the callable value and shares its environment.
Reference counting releases an acyclic environment when its final owner is released.
A cycle through a mutable capture can retain its environment. The runtime supports
explicit local collection through its internal maintenance interface; it preserves
cycles reachable from execution, host or debugger roots and removes unreachable
cell edges. Automatic collection is not enabled. See
[runtime ownership](../../program-structure/runtime-ownership.md#explicit-cycle-reclamation).

Closure equality and ordering are not defined. Test for assignment with the existing
optional-value facilities when needed.

Recursive anonymous closures are not implicit. Use a named nested routine or an
explicitly declared callable binding that is initialized before invocation.

## Concurrency

An immutable capture environment may cross a task boundary. A closure that contains a
mutable capture is **task-bound** and cannot be used as the callable of `go`, sent to
another task, or returned through a task result. Capturing another task-bound callable
also makes the outer closure task-bound (the mutable cells are still reachable).
Runtime closure construction follows the complete capture graph, including cells or
task-bound functions inside arrays, dictionaries, records and payloads. An immutable
outer wrapper does not change the ownership of its contents. Cell reads and writes
check the logical task's ownership before accessing storage; replacing a cell with
a foreign local graph fails without changing its previous value.

The runtime checks complete graphs before every task start, retained retry, channel
send and published task result, including arguments, captures and bound receivers.
This also covers values returned by ordinary helpers whose capture state is not
known to the compiler. A rejected start admits no child; a rejected send leaves
the channel unchanged; a forbidden task result completes through the task-failure
path. Argument expressions already evaluated keep their effects.

The checker distinguishes proven transferable values, known task-bound values,
and values whose capture state is unknown. It reports FP3016 for known task-bound
callables, arguments, channel values and task results at the transfer operation.
This includes locally resolved factory results when their return capability is
known. Returning a local closure to an ordinary caller remains valid.

Branch and loop joins retain known local state, as do field/index writes and
aggregate wrappers. A whole-binding assignment replaces the previous capability;
overwriting one field cannot prove that all other fields are transferable.
Scalar projections remain transferable. Callable parameters and unresolved or
imported factory results stay unknown without proof and proceed to the runtime
guard. Uncertainty alone is not a compile-time error.

An immutable global containing a local closure remains usable by its owning task,
but a foreign task cannot load that graph. Selection callbacks stay on their
selecting task and may retain its mutable captures.

Extracting a callable with `const Name` in a `case` pattern preserves the
matched value's known task-bound state, including nested payloads and uses in
guards or closures that capture the extracted callable. Binding a scalar
payload does not make that scalar task-bound. An `is` test never binds names;
use `case` to extract a payload.

```pascal id=example-5-valid fragment context=body
// Accepted: immutable capture
const N: integer := 3;
const Work: function(): integer := function(): integer begin
  return N * 2;
end function;

scope
  const Handle: task of integer := go Work();
  discard Handle;
end scope;
```

```pascal id=example-5 fragment context=mutable-closure error=FP3016
// Rejected: mutable capture
var Count: integer := 0;
const Inc: procedure() := procedure() begin
  Count := Count + 1;
end procedure;

scope
  discard go Inc();
end scope; // Compile-time error
```

```pascal id=example-5-nested fragment context=body error=FP3016

var Count: integer := 0;
const Inc: procedure() := procedure() begin
  Count := Count + 1;
end procedure;

// Rejected: nested task-bound capture
const Outer: procedure() := procedure() begin
  Inc();
end procedure;

scope
  discard go Outer();
end scope; // Compile-time error — Outer captures task-bound Inc
```

Workers also cannot access mutable program/unit globals through named helpers
or callbacks. Known dependencies are checked at starts (FP3042). Dynamic targets
retain mandatory VM access checks. Pass ordinary values into the worker, mutate
its own local copies, or use channels. Discarding a captured callable is ordinary
value disposal and does not transfer or invoke it.

## Panic and cleanup

Unwinding through a closure releases ordinary locals and closure values using the same
managed-value rules as a normal routine. A panic in a closure preserves the original
diagnostic.

## See also

- [First-class functions](first-class.md)
- [Nested functions](nested.md)
- [Function types](function-types.md)
- [Concurrency](../concurrency/README.md)
- [Runtime ownership for Rust hosts](../../program-structure/runtime-ownership.md)
