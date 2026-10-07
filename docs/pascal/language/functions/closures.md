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

```pascal
Apply(function(Value: integer): integer begin
  return Value * 2;
end function);
```

Here the `;` after `)` terminates the call statement.

```pascal
var Count: integer := 0;

const Increment: procedure() := procedure() begin
  Count := Count + 1;
end procedure;
const AddBase: function(Value: integer): integer := function(Value: integer): integer begin
  return Count + Value;
end function;
```

Closures may be stored in variables and records, passed as arguments, returned from
routines, and invoked through ordinary call syntax.

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
the same cell. The cell survives until the final closure that references it is released.

```pascal
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

```pascal
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

## Lifetime and equality

Creating or copying a closure copies the callable value and shares its environment.
Releasing the final copy releases the environment.

Closure equality and ordering are not defined. Test for assignment with the existing
optional-value facilities when needed.

Recursive anonymous closures are not implicit. Use a named nested routine or an
explicitly declared callable binding that is initialized before invocation.

## Concurrency

An immutable capture environment may cross a task boundary. A closure that contains a
mutable capture is **task-bound** and cannot be used as the callable of `go`, sent to
another task, or returned through a task result. Capturing another task-bound callable
also makes the outer closure task-bound (the mutable cells are still reachable).

```pascal
// Accepted: immutable capture
const N: integer := 3;
const Work: function(): integer := function(): integer begin
  return N * 2;
end function;
const Handle: task := go Work();
// Rejected: mutable capture
var Count: integer := 0;
const Inc: procedure() := procedure() begin
  Count := Count + 1;
end procedure;

go Inc(); // Compile-time error
// Rejected: nested task-bound capture
const Outer: procedure() := procedure() begin
  Inc();
end procedure;

go Outer(); // Compile-time error — Outer captures task-bound Inc
```

## Panic and cleanup

Unwinding through a closure releases ordinary locals and closure values using the same
managed-value rules as a normal routine. A panic in a closure preserves the original
diagnostic.

## See also

- [First-class functions](first-class.md)
- [Nested functions](nested.md)
- [Function types](function-types.md)
- [Concurrency](../concurrency/README.md)
