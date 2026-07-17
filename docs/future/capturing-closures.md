# Capturing closures

## Status

Planned language feature. This document defines the intended language contract and an
implementation order. Current behavior remains documented under `docs/pascal/`.

Current next step: implement closure literals and closure environments after the expression
postfix-chaining work has finished. Postfix chaining is independent and is not part of this plan.

## Motivation

Functional Pascal already supports function and procedure types, named routines as values, and
nested routines that read their lexical scope while their parent invocation is active. It does not
yet provide a callable value that owns an enclosing environment and can outlive the invocation that
created it.

Long-lived callbacks therefore require global or unit-owned state. Capturing closures make event
handlers, collection operations, asynchronous continuations, and small configurable algorithms
local and composable.

## Decision

Add anonymous function and procedure expressions and allow a referenced nested named routine to
become a capturing closure. The resulting value uses the existing function or procedure type whose
signature matches the closure.

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

A closure literal is an expression. Its final `end` belongs to the expression; the surrounding
declaration, assignment, argument list, or record field supplies any required separator.

Contextual typing is preferred, but a closure is also self-typed from its declared parameters and
function result. Parameter and result annotations are mandatory in version 1.

```pascal
Consume(
  procedure(Value: integer)
  begin
    WriteLn(Value)
  end
);
```

## Capture rules

Capture is lexical and automatic. A name is captured when the closure body refers to a local,
parameter, or enclosing capture that is not declared by the closure itself.

| Binding | Capture behavior |
| --- | --- |
| Immutable local or value parameter | Capture its value when the closure is created. |
| `mutable var` local | Capture one shared mutable cell. |
| Enclosing closure capture | Reuse the same value or mutable cell. |
| Unit or program variable | Resolve normally; it is not stored in the closure environment. |
| Routine, static function, or constant | Resolve normally; it is not stored as runtime data. |

All closures created by one activation and capturing the same mutable local observe the same cell.
The cell survives until the final closure that references it is released.

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

Capturing a mutable parameter requires the parameter target to be promoted to the same managed cell
model. The compiler must not retain an unchecked stack reference. If a parameter mode cannot be
promoted safely, capturing it is a compile-time error with a diagnostic naming the parameter.

There is no capture-list syntax in version 1. Immutability already makes the important distinction
visible at the variable declaration, and a second declaration site would add noise without adding
safety.

## Lifetime and value semantics

A closure consists of a code identity and a managed environment. Creating or copying a closure
copies the callable value and shares its environment. Releasing the final copy releases the
environment and every value it owns.

Closures may be:

- stored in variables and records;
- passed to and returned from routines;
- stored by standard-library facilities such as event slots;
- invoked through the existing function and procedure call syntax.

Closure equality and ordering are not defined. A callable may only be tested for assignment through
the existing optional-value facilities or through the event operations defined in
[events-and-bound-methods.md](events-and-bound-methods.md).

Recursive anonymous closures are not implicit. Recursion uses a named nested routine or an
explicitly declared callable binding whose definite-assignment rules prove that it is initialized
before invocation.

## Concurrency

An immutable capture environment may cross a task boundary. A closure containing a mutable capture
is task-bound and cannot be used as the callable of `go`, sent to another task, or returned through
a task result.

This restriction is part of the callable value, not merely the closure expression. Sema records the
capability on function values, and the runtime retains a defensive check when static provenance has
been erased. A diagnostic must identify the mutable captured binding and suggest passing an
immutable input value instead.

Main-task queues may accept a task-bound closure when it is created and invoked on that same task.
Worker tasks may post only transferable closures back to the main task.

## Panic and cleanup

Unwinding through a closure releases ordinary locals and closure values using the same managed-value
rules as a normal routine. A panic in a closure preserves the original diagnostic and must not be
replaced by environment cleanup errors.

## Scope boundaries

Version 1 does not add:

- implicit currying or partial application;
- variadic closure parameters;
- closure equality;
- weak captures;
- user-written capture lists;
- object instances, inheritance, or an implicit `this` value;
- a new overload system;
- syntax or lowering for expression postfix chaining.

Bound record methods are a separate callable-producing expression described in
[events-and-bound-methods.md](events-and-bound-methods.md).

## Expected implementation shape

Keep parsing, capture analysis, lowering, runtime representation, and tests in focused modules. The
exact paths may follow the owning crate's current organization when implementation begins.

```text
crates/fpas-parser/src/
  ast/expr.rs                         — add closure expression nodes
  parser/expr/closure.rs              — NEW: anonymous routine expressions
  tests/expr/closures.rs              — NEW: syntax and recovery

crates/fpas-sema/src/check/
  expr/closure.rs                     — NEW: signature and body checking
  closures/capture.rs                 — NEW: lexical capture analysis
  closures/capability.rs              — NEW: task-transfer classification

crates/fpas-compiler/src/compiler/
  expr/closure.rs                     — NEW: environment construction
  closures/environment.rs             — NEW: capture layout and nested access

crates/fpas-bytecode/src/
  closure.rs                          — NEW: closure metadata and opcodes

crates/fpas-vm/src/
  value/closure.rs                    — NEW: managed callable environment
  vm/execute/closure.rs               — NEW: construction and invocation
```

Do not put capture analysis into general name resolution or encode environments as untyped arrays
inside unrelated VM call code.

## Implementation order

1. Add closure AST nodes, parser recovery, formatter support, and round-trip tests.
2. Type-check anonymous signatures and bodies without captures.
3. Analyze immutable captures and lower managed environments.
4. Add mutable-cell promotion and shared-cell tests.
5. Convert referenced nested routines into escaping closure values when required.
6. Add transferable versus task-bound callable classification.
7. Add VM cleanup, panic, and lifetime canaries.
8. Document implemented behavior under `docs/pascal/` and remove this plan when complete.

## Required tests

- anonymous procedure and function literals in assignments and arguments;
- returned closure outliving its creating routine;
- immutable local and parameter captures;
- shared mutation observed by two sibling closures;
- independent environments from separate outer invocations;
- nested closure capturing an enclosing capture;
- capture of strings, arrays, records, and function values;
- named nested routine escaping as a function value;
- wrong parameter or result type diagnostics;
- closure environment release on normal return and panic;
- immutable closure accepted across a task boundary;
- mutable closure rejected across a task boundary;
- formatter round trips for compact and multiline closures.

## Acceptance criteria

- examples in this document compile and preserve lexical state;
- closures can be stored, copied, returned, and invoked through existing callable types;
- mutable captures use managed cells and never retain raw stack references;
- capture and argument evaluation order is deterministic;
- task transfer rules produce specific diagnostics;
- named non-capturing routine values retain their current representation and behavior;
- current language documentation and grammar describe the feature only after it works;
- `cargo fmt`, `cargo build --workspace`, `cargo test --workspace`, Clippy with warnings denied,
  FPAS regression tests, and `git diff --check` pass.

## Plan lifecycle

Keep this document under `docs/future/` while any acceptance criterion is incomplete. Once the
implemented grammar, language documentation, and tests are authoritative, remove this plan and its
future index entry.
