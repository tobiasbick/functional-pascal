# Std.Tui2 event loop and routing

## Terminal boundary

`Std.Console` remains the only terminal-input boundary. Std.Tui2 reads unified `Std.Console.Event` values through `ReadEventTimeout` after enabling raw mode and the needed terminal features.

The Rust runtime maps terminal input to `Std.Console.Event`; it does not select views, resolve commands, or invoke application handlers for Std.Tui2.

## Application loop

Implemented headless core: `RunIterations(IterationCount, DeltaMilliseconds)` starts the application
once, executes a deterministic maximum number of iterations, and remains open when the budget is
exhausted. Each iteration drains posts, completes dirty desktop layout, runs `OnTick`, and drains
posts again. Pending custom-view resize notifications run after layout and before `OnTick`. `Quit`
stops at a callback boundary and performs orderly close. `ResizeForTest` updates the application and
desktop extents, so the next iteration performs the invalidated layout.

Interactive `Run` uses the same phase order after `AcquireInteractiveTerminal`, flushes the painted
surface through `Std.Console` cells, and replaces injected input with one `ReadEventTimeout(16)`
value per iteration.

Headless input is also implemented. `App.Input` queues keys and pointer values FIFO, and an iteration
routes at most one value after paint and before `OnTick`. Keys target the focused eligible custom
view; pointer values target capture first and otherwise the topmost hit. Unconsumed values reach the
application input fallback.

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
7. The action's `OnExecute` event runs synchronously with the originating view as its source.
8. The control's direct semantic event runs after the action when the source remains live.
9. Typed value-change events run after state mutation.
10. Unhandled key and mouse events receive the original input.

Raw input events return `boolean`: `true` consumes the event and `false` leaves it available to the
next routing step. Action and typed change events are semantic callbacks and do not participate in
raw input propagation. See [events-and-actions.md](events-and-actions.md).

## Key representation

Keyboard matching uses `Std.Console.KeyKind` and modifier flags. Non-character keys such as Enter
and Escape are matched by their named key kinds. Live terminal mapping stores the null character in
their string field, so routing code must not match them as `Chr(13)` or `Chr(27)`.

Normal routing and every callback run on the main task. Worker integration uses the typed queue in [runtime-boundary.md](runtime-boundary.md).
