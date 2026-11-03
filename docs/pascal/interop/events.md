# CLR events

Imported CLR event subscription/removal is [implemented](../../implementation/events.md).

```pascal fragment
const Handler: Timers.ElapsedEventHandler :=
  procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
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

uses System.Console as Console,
  System.Threading as Threading,
  System.Timers as Timers;

begin
  using const Timer: Timers.Timer := new Timers.Timer(100.0);
  const Handler: Timers.ElapsedEventHandler :=
    procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
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

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
