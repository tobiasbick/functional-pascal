# Std.Tui2 event loop and routing

## Terminal boundary

`Std.Console` remains the only terminal-input boundary. Std.Tui2 reads unified `Std.Console.Event` values through `ReadEventTimeout` after enabling raw mode and the needed terminal features.

The Rust runtime maps terminal input to `Std.Console.Event`; it does not select views, resolve commands, or invoke application handlers for Std.Tui2.

## Application loop

The initial application loop has this shape:

```text
enable terminal mode
repeat while running
  drain posted main-task callbacks
  complete invalid layout
  paint invalid regions
  read an event with a bounded timeout
  route an event when one arrived
  run one application tick
  drain posted main-task callbacks
restore terminal mode
```

Application `OnStart`, `OnStop`, and view lifecycle hooks run at defined points around this loop. See [view-lifecycle.md](view-lifecycle.md).

## Routing order

For each event, Std.Tui2 applies this order:

1. Resize updates application geometry and invalidates affected views.
2. A modal root limits routing to its modal subtree.
3. Pointer capture receives matching mouse events first.
4. Mouse events hit-test the topmost eligible view.
5. Key events go to the focused eligible view.
6. An eligible control may consume the input and activate its bound `TuiAction`.
7. The action handler runs synchronously with the originating view as its source.
8. Typed control changes notify their registered handler after state mutation.
9. Unhandled key and mouse callbacks receive the original event.

Raw input callbacks return `boolean`: `true` consumes the event and `false` leaves it available to the next routing step. Action handlers and typed change notifications are semantic callbacks and do not participate in raw input propagation. See [actions-and-handlers.md](actions-and-handlers.md).

## Key representation

Keyboard matching uses `Std.Console.KeyKind` and modifier flags. Non-character keys such as Enter and Escape are matched by their named key kinds; their character field is empty.

Normal routing and every callback run on the main task. Worker integration uses the typed queue in [runtime-boundary.md](runtime-boundary.md).
