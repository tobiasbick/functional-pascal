# CLR events

Use subscribe/unsubscribe with events declared by referenced .NET assemblies
and ordinary CLR delegates.

```pascal fragment context=events
const Handler: Timers.ElapsedEventHandler := procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
  Console.WriteLine('Tick');
end procedure;

subscribe Timer.Elapsed with Handler;
unsubscribe Timer.Elapsed with Handler;
```

`subscribe EventReference with Delegate;` invokes the CLR add accessor.
`unsubscribe` invokes the remove accessor. Evaluate the event receiver before
the delegate, once each. Event references are special member selections, not
ordinary readable delegate values. Accessibility is checked on each accessor.

Store the delegate when later removal must use the same instance/target. The
compiler does not turn repeated lambda evaluation into a stable identity or
automatically remove subscriptions at scope exit. There is no source event
declaration in the initial language.

## Example

```pascal
program Events;

uses
  System.Console as Console,
  System.Threading as Threading,
  System.Timers as Timers;

begin
  using const Timer: Timers.Timer := new Timers.Timer(100.0);
  const Handler: Timers.ElapsedEventHandler := procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
    Console.WriteLine('Tick');
  end procedure;

  subscribe Timer.Elapsed with Handler;
  Timer.Start();
  Threading.Thread.Sleep(350);
  Timer.Stop();
  unsubscribe Timer.Elapsed with Handler;
end;

end program;
```

Tick timing/count is nondeterministic. Unsubscription does not wait for an event
handler already dispatched by the CLR timer. This example illustrates event
binding, not a synchronization or exact-timing guarantee.

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

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
