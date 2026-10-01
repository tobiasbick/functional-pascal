# Stage 5: Effects and tasks

The [contract](../language-contract.md) fixes semantics before implementation.
The caller-mutation slice integrates with [stage 4](04-functional-core.md);
purity checking and task scopes build on its callable/data facilities.

## Explicit caller mutation

`procedure Increase(var Value: integer);` permits `Increase(var Counter);`.
The same mode is part of function and procedure types, so indirect calls require
the same marker. A var argument is a mutable binding or a writable field/element
path rooted in one. Const bindings, temporaries, and read-only parameters are
invalid roots. A var parameter can be forwarded with `var`.

Evaluate the target and arguments left to right. Evaluate each var path's root
and indices once; check access/bounds before entering the callee. Mutations inside
the callee are immediately visible through that reference, with no rollback or
copy-back phase. A panic or propagated Result error preserves changes already
made. Changes during argument evaluation also remain if a later argument fails.

Two var arguments cannot share a root, even for different fields or indices.
Resolve forwarded aliases and captured cells by storage identity, not spelling;
use a runtime check when static proof cannot rule out overlap. During argument
evaluation, reserve resolved var paths against mutation through another alias;
read-only snapshots remain allowed. On callee entry the reference becomes
exclusive: other aliases cannot read or write that root. Forwarding reborrows
the authorized reference and suspends its parent's access until the call returns.
This covers callbacks, global/captured cells, and structural resizing during path
evaluation. It is not a user-visible borrow syntax or lifetime-parameter system.

Read-only value arguments are snapshots evaluated at their written position.
For example, passing a value copy and a var reference to the same root is allowed;
the snapshot cannot observe later mutation of that value storage. Resource identity
within the snapshot still obeys the ordinary resource rules.

Audit every intrinsic that replaces caller storage, including Push/Pop-style
collection APIs. Such calls require `var` like ordinary routines. Operations on a
resource's state use its handle without pretending to replace the handle binding.
Never migrate a locally reassigned `mutable` parameter into caller mutation.

## Pure functions and higher-order calls

Use `pure function` for both declarations and callable types. Pure functions have
no var parameters; may call only pure functions; and may capture/read only
immutable, resource-free data and pure callable values. Pure parameters and results
must likewise be resource-free data or pure callable values, recursively through
records, variants, and collections. Generic pure routines must enforce that
restriction on concrete instantiations, in addition to operation constraints.

Local var data is allowed if its state does not escape. A pure function cannot
return a stateful closure or store a captured mutable cell elsewhere. It may panic
or fail to terminate; purity does not prove totality. Pure procedures are excluded:
procedures describe actions and have no returned value.

Target example (the callback capability is part of the signature):

```pascal
pure function ApplyTwice(
  F: pure function(Value: integer): integer;
  Value: integer
): integer;
begin
  return F(F(Value));
end function;
```

A pure callable is accepted by an ordinary callable parameter, but its purity
guarantee is unavailable through that less restrictive type. An ordinary callable
cannot satisfy a pure parameter. Parameter names never affect this relation.
No effect polymorphism or inferred purity is added. Functional collection APIs
such as Map accept pure callbacks; action-oriented traversal uses ordinary
procedures. Do not create duplicate pure/impure overloads of the same routine.

Standard-library purity metadata must be audited against implementations. Record
defaults may invoke only pure operations and obey stage 1's evaluation rules.
Contracts in stage 6 use this same checker, not a separate whitelist of calls.

## Structured task scopes

Every spawn occurs lexically inside `scope ... end scope;`, even at program entry.
A nested routine cannot inherit the scope merely because its declaration or call
appears inside one; it opens a scope in its own body when it spawns.

`go Procedure(...)` is a statement with no result handle. `go Function(...)` is
an expression returning `task of (T)`. Both register with the innermost scope.
The procedure form has no hidden public unit-result type or bare-task exception.
Calling `go` through a stored callable follows its declared function/procedure kind.

```pascal
uses Std.Tasks as Tasks;

procedure Run();
begin
  scope
    const Job := go Compute(42);
    const Value := Tasks.Wait(Job);
    Store(Value);
  end scope;
end procedure;
```

This fragment assumes ordinary declarations for Compute and Store; it illustrates
the target lifecycle, not a runnable current program.

Handles are scope-bound even when copied into collections or closures. Synchronous
helpers can receive them only if analysis proves they do not retain, return, spawn
with, or send them. Imported helper contracts must carry this verified property;
unknown indirect calls are conservatively rejected. Resource stores/channel sends
cannot carry task handles. A child cannot return a handle from its own inner scope.
Task-bound mutable closures cannot cross task boundaries by any container route.

Wait returns the function result or propagates its panic, and marks that result
observed. Repeated waits return the stored value under ordinary copy/identity
rules; they do not execute the task again. A barrier waits without observing
results. On normal scope completion every function-task result must have been
observed; otherwise report a runtime misuse diagnostic after joining all children.
Statically obvious dropped handles should be diagnosed earlier. Abnormal scope
exit cancels/joins children without requiring observation of their results.
`discard Tasks.Wait(Job);` is valid for an ordinary result; discarding Job is not.

Normal exit waits without cancellation. A boundary-crossing return, break,
continue, `try` propagation, or panic requests cancellation then waits. A break
inside a loop contained by the scope does not exit that scope. The first observed
child panic also requests sibling cancellation. Further panics are attached as
secondary diagnostics. If the scope body already panicked, retain that primary
panic; otherwise a child panic takes precedence over a pending return/error after
cleanup. An ordinary returned Result.Error never triggers sibling cancellation.

Cancellation is cooperative. Blocking host operations need cancellation-aware
wakeup or a documented inability to finish until the operation returns. A scope
cannot claim completion while its children still run. Parent cancellation must
propagate through nested scopes; cancellation itself is not a sibling panic.

All public spawning routes obey this model. Replace StartTaskInGroup,
StartSupervisedTask, and similar independent-lifetime entry points with ordinary
scope-based application code where needed. Internal scheduler/group machinery may
be reused. Do not keep an alternate user-owned group lifetime or hidden detached
escape hatch. Supervision/retry is an ordinary policy inside an owning scope.

## Work and acceptance

- [ ] Audit mutation intrinsics, mutable-parameter behavior, alias paths, callback
  captures, and exception/error exits; coordinate the binding migration.
- [ ] Implement var modes through signatures, callable types, lowering, runtime
  references, diagnostics, and every caller-mutating standard API.
- [ ] Implement pure declarations/types, transitive capture checks, and verified
  standard-library metadata; cover higher-order and generic instantiation cases.
- [ ] Audit all spawn APIs and task transport. Implement scope ownership, helper
  escape analysis, observation, cancellation, joining, and failure arbitration.
- [ ] Migrate detached/group-based consumers, including servers and test runners.
  Replace dependencies on background lifetime with an explicit enclosing scope.
- [ ] Update parameter/capture/function-type, concurrency, and Std.Tasks docs;
  remove stale APIs and generated declarations across compiler/runtime/editor.
- [ ] Test var path order/aliasing, partial mutation on failure, pure callbacks,
  indirect escape attempts, both go forms, repeated waits, missing observation,
  nested scopes, every exit path, child panics, returned Result errors, and blocked
  child cancellation. Test absence of surviving children after completed exit.

Acceptance: caller mutation is explicit through direct and indirect calls; purity
survives higher-order composition; every spawned task has one owner and every
completed scope has joined its children. Results and panics follow separate rules.

Owners: sema and callable metadata, compiler/IR/bytecode, VM task scheduling and
host integration, `fpas-std`, std source/registries, CLI runner, formatter, and LSP.

Status: contracts specified; implementation and migration pending.
Next: audit existing mutation paths with stage 4, then test one pure higher-order
routine and one scope through the actual compiler/VM path before broad migration.
