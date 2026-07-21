# Std.Tui3 event loop

## Terminal boundary

`Std.Console` remains the only terminal-input boundary. Std.Tui3 reads unified
`Std.Console.Event` values through `ReadEventTimeout` after enabling raw mode and the
needed terminal features.

The Rust runtime maps terminal input to `Std.Console.Event`. It does not select widgets,
resolve actions, or call application `Update`.

## Loop shape

```text
acquire terminal (interactive) or open headless host
Model := Init
Tree := View(Model)
layout Tree for current Size
paint Tree to working Surface
flush Surface (interactive only)
repeat while running
  if message queue is empty then
    read one input, injected message, Resize, or virtual/live Tick
    route it against the previous laid-out Tree into zero or more queued TuiMsg values
  if message queue is still empty then
    continue
  Msg := dequeue exactly one TuiMsg
  Cmd := TuiCmd.None
  Model := Update(Model, Msg, Cmd)
  if Cmd = Quit then
    running := false
  else
    Tree := View(Model)
    layout Tree for current Size
    paint Tree to working Surface
    flush Surface (interactive only)
restore terminal (interactive)
```

The initial render is mandatory and happens before input can be targeted. `RunIterations` counts
message-processing iterations; the initial render does not consume that budget. Pending messages
are processed before another external event is read. A `Quit` command stops before another
`View`/paint/flush pass. Headless `RunIterations` uses the same phase order with injected messages
and a virtual clock; it does not change terminal modes.

## Message production order

Routing one external input may enqueue more than one message. Their order is:

1. `Resize` updates the host size before `Update` when the terminal changed.
2. If `View`'s previous laid-out tree marked a modal dialog subtree, pointer and key
   targeting is limited to that subtree.
3. Pointer capture (if enabled) receives matching pointer events first.
4. Pointer hits the topmost eligible interactive element; a focus change is queued before the
   control's `Action` or controlled-value message.
5. Keys go to the focused interactive element when present; otherwise to application-level
   key handling via `TuiMsg.Key`.
6. An input, check box, list, or scroll control produces its dedicated next-value message; it does
   not mutate hidden persistent widget state.
7. Shortcut tables (optional later) may translate keys into `TuiMsg.Action` before the raw key
   reaches the application.
8. Unhandled keys and pointer input remain `TuiMsg.Key` or `TuiMsg.Pointer` for `Update`.

There is no widget `OnClick` after action dispatch. Activation **is** the message.

## Key representation

Keyboard matching uses `Std.Console.KeyKind` and modifier flags. Non-character keys such
as Enter and Escape match by named kinds. Live terminal mapping may store a null character
in their string field; routing must not treat them as `Chr(13)` or `Chr(27)`.

## Tick

When ticking is enabled, the loop synthesizes `TuiMsg.Tick(DeltaMilliseconds)` only when no queued,
injected, resize, or terminal-input message is available for that iteration. `Update` may ignore
ticks.

## Relationship to Tui2 routing

Tui2 routed events into live view handlers and registry-backed actions. Tui3 routes events
into **messages** and a single `Update`. Hit-testing still needs a laid-out tree; that tree
comes from the last `View` pass, not from retained object identity.
