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
repeat while running
  drain posted main-task callbacks / completed commands → TuiMsg
  Msg := next input or Tick or injected test message
  (Model, Cmd) := Update(Model, Msg)
  execute Cmd (may enqueue Quit or further messages)
  Tree := View(Model)
  layout Tree for current Size
  paint Tree to Surface
  flush Surface (interactive only)
restore terminal (interactive)
```

Headless `RunIterations` uses the same phase order with injected messages and a virtual
clock. It does not change terminal modes.

## Message production order

For one iteration that handles input:

1. `Resize` updates the host size before `Update` when the terminal changed.
2. If `View`'s previous laid-out tree marked a modal dialog subtree, pointer and key
   targeting is limited to that subtree.
3. Pointer capture (if enabled) receives matching pointer events first.
4. Pointer hits the topmost eligible interactive element; a press may produce focus-move
   messages and/or `TuiMsg.Action`.
5. Keys go to the focused interactive element when present; otherwise to application-level
   key handling via `TuiMsg.Key`.
6. Shortcut tables (optional later) may translate keys into `TuiMsg.Action` before the raw
   key reaches the application.
7. Unhandled keys remain `TuiMsg.Key` for `Update`.

There is no widget `OnClick` after action dispatch. Activation **is** the message.

## Key representation

Keyboard matching uses `Std.Console.KeyKind` and modifier flags. Non-character keys such
as Enter and Escape match by named kinds. Live terminal mapping may store a null character
in their string field; routing must not treat them as `Chr(13)` or `Chr(27)`.

## Tick

When ticking is enabled, each iteration synthesizes `TuiMsg.Tick(DeltaMilliseconds)` if no
higher-priority quit occurred. `Update` may ignore ticks.

## Relationship to Tui2 routing

Tui2 routed events into live view handlers and registry-backed actions. Tui3 routes events
into **messages** and a single `Update`. Hit-testing still needs a laid-out tree; that tree
comes from the last `View` pass, not from retained object identity.
