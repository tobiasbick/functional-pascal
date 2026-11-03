# Imported CLR events

The compiler implements the existing [event syntax](../pascal/interop/events.md)
for events declared by referenced .NET assemblies. The
[collection-events example](../../examples/events/README.md) subscribes a capturing
delegate to BindingList<int>.ListChanged and removes it before a second change.

```pascal
const Handler: Component.ListChangedEventHandler :=
  procedure(Sender: object?; Change: Component.ListChangedEventArgs) begin
    Console.WriteLine(Change.ListChangedType);
  end procedure;
subscribe Values.ListChanged with Handler;
Values.Add(42);
unsubscribe Values.ListChanged with Handler;
```

## Lookup and binding

Resolve actual Event/MethodSemantics metadata rather than guessing add_/remove_
names or treating an event as a delegate field. Check the selected accessor
independently: it must be public, have the requested instance/static shape,
return void, and take one supported by-value delegate argument matching the
event's CLR type. Missing/inaccessible/unsupported add access does not prevent
using a supported remove accessor, and conversely.

Lookup includes inherited class/interface events and substituted generic owners,
including generic parameter constraints. Derived declarations hide base members;
an inaccessible event or a delegate field does not expose a hidden base event.
Unrelated inherited event selections are ambiguous; explicitly select
the desired declaring interface rather than relying on traversal order.

Delegate values, compatible typed lambdas, and named/overloaded routines use the
ordinary delegate binding, inference, conversion, and nullable rules. Nullable
event receivers require validation or coalescing. The handler is an ordinary
accessor argument, including its imported nullable parameter annotations.

Events cannot be read, assigned, invoked, or passed as ordinary delegate values.
Properties and delegate-valued fields are not event references. FP3034 diagnoses
event selection/access errors. Existing overload/delegate/type, nullable, out-flow,
and async reference-lifetime diagnostics still apply to operands.

## Evaluation and execution

Bind subscriptions to ordinary bound accessor calls and use the existing CIL
emitter/async lowering. Evaluate receiver storage before the handler, once each
in written order, then invoke the selected accessor. Preserve virtual/interface
dispatch and generic owner signatures. Static events have no runtime receiver.
Member chains through static properties and global fields remain value accesses.

Writable struct locals, ref parameters, fields, and array elements retain their
actual storage. Constrained generic calls do not mutate a boxed replacement.
Read-only struct receivers retain ordinary defensive-copy behavior. A managed
receiver reference cannot remain live across an awaited handler; evaluate the
handler before selecting writable struct storage in that case.

Reference receivers and delegates may survive real async suspension. Receiver,
handler, and accessor exceptions propagate in ordinary order; a failed operand
prevents invocation. Accessor calls invalidate mutable-capture nullable facts
because custom accessors may execute callbacks. Existing flow/capture traversal
includes both operands, including nested routines and loop ref/out writes.

Retain the delegate when removal requires the same target/instance. Re-evaluating
a lambda does not invent a stable handler identity. No automatic removal occurs
at scope exit; use explicit try/finally when the subscription lifetime needs
cleanup, including async finally.

## Verification and limits

Compiler tests cover BCL events, delegate targets, invalid operands, ordinary
event value operations, nullable capture invalidation, and out-parameter reads.
Metadata tests independently exercise inaccessible/missing/unsupported accessors
and accessor names that differ from the conventional spelling.

Mixed C#/FPAS SDK projects execute generic/inherited/static/virtual/interface
events, duplicate subscriptions and one-at-a-time removal, captured delegates,
sync/async finally, operand/accessor exceptions, true async suspension, writable
and read-only struct receivers, and constrained generic storage. Invalid imported
event operations are also checked through the CLI. The executable collection
example uses the actual .NET collection event; the contract fixture runs in
Debug and Release.

Source event declarations and source interface declarations remain outside the
original baseline. Continue with the existing work in
[language gaps](../future/language-gaps.md#finish-the-existing-baseline-first).
