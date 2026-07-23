# Capturing closures

Anonymous `function` / `procedure` expressions create callable values that own a
managed lexical environment. The value uses the existing function or procedure type
whose signature matches the closure.

Parameter and result annotations are mandatory. The final `end` belongs to the
expression; surrounding syntax supplies any separator.

```pascal
mutable var Count: integer := 0;

var Increment: procedure() :=
  procedure()
  begin
    Count := Count + 1
  end;

var AddBase: function(Value: integer): integer :=
  function(Value: integer): integer
  begin
    return Count + Value
  end;
```

Closures may be stored in variables and records, passed as arguments, returned from
routines, and invoked through ordinary call syntax.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`closure_expr`).

## Capture rules

Capture is lexical and automatic. A name is captured when the closure body refers to
a local, parameter, or enclosing capture that is not declared by the closure itself.

| Binding | Capture behavior |
| --- | --- |
| Immutable local or value parameter | Capture its value when the closure is created. |
| `mutable var` local or `mutable` parameter | Capture one shared mutable cell. |
| Enclosing closure capture | Reuse the same value or mutable cell. |
| Unit or program variable | Resolve normally; not stored in the closure environment. |
| Routine, static record routine, or constant | Resolve normally; not stored as runtime data. |

All closures created by one activation and capturing the same mutable local observe
the same cell. The cell survives until the final closure that references it is released.

```pascal
function Counter(): function(): integer;
begin
  mutable var Value: integer := 0;
  return function(): integer
  begin
    Value := Value + 1;
    return Value
  end
end;
```

There is no capture-list syntax. Immutability is declared at the variable (or
parameter) site.

## Named nested routines

A nested named routine that refers to enclosing locals becomes a capturing closure
when it is used as a first-class value (assigned, returned, or passed):

```pascal
function MakeAdder(Base: integer): function(Value: integer): integer;
  function Add(Value: integer): integer;
  begin
    return Base + Value
  end;
begin
  return Add
end;
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
{ Accepted: immutable capture }
var N: integer := 3;
var Work: function(): integer :=
  function(): integer
  begin
    return N * 2
  end;
var Handle: task := go Work();

{ Rejected: mutable capture }
mutable var Count: integer := 0;
var Inc: procedure() :=
  procedure()
  begin
    Count := Count + 1
  end;
go Inc();  { compile-time error }

{ Rejected: nested task-bound capture }
var Outer: procedure() :=
  procedure()
  begin
    Inc()
  end;
go Outer();  { compile-time error — Outer captures task-bound Inc }
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
