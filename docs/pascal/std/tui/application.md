# `Std.Tui` application host

## Update and View

The headless and terminal hosts accept this callable shape:

```pascal
function Update(
  State: AppModel;
  Msg: TuiMsg;
  Cmd: TuiCmdOutput
): AppModel;

function View(State: AppModel): TuiElement;
```

Set a command explicitly when needed:

```pascal
TuiMsg.QuitRequested:
begin
  Cmd.Set(TuiCmd.Quit);
  return State
end
```

`TuiCmdOutput` is a host-owned output capability. The host resets it to
`NoCommand` before every `Update`, reads it immediately afterwards, and stops
on `Quit` before calling `View` or painting again.

`Cmd.SetPalette(Palette)` replaces the active color palette before the next
paint. `TuiCmd` remains a closed scalar command enum; the host-owned
`TuiCmdOutput` carries the palette payload separately.

## Background messages and owned work

A background-enabled application uses a caller-created bounded `channel of TMessage` and a second
update callback:

```pascal
function UpdateApplication(
  State: AppModel;
  Message: AppMessage;
  Cmd: TuiCmdOutput
): AppModel;

var Inbox: channel of AppMessage := CreateChannel(32);
var Final: AppModel := TuiApplication.RunWithBackground(
  Initial, Inbox, Update, UpdateApplication, View
)
```

`RunWithBackgroundAndPalette` adds the same palette argument as `RunWithPalette`. The host takes
ownership of `Inbox` for the run and closes it during shutdown. A worker normally publishes with
`SendWithCancellation(Inbox, Message, Token)` so a full queue applies bounded backpressure and
shutdown wakes a blocked send. `TrySend` provides explicit rejection instead: `Ok(false)` means the
open queue is full, while `Error` means it is closed. No application message is silently dropped.

The initial frame is rendered before the background-enabled host delivers `TuiMsg.Started`.
`Update` can use that message to queue initial work:

```pascal
TuiMsg.Started:
begin
  Cmd.StartBackground(1, LoadData)
end
```

The fixed work signature is:

```pascal
function Work(Token: CancellationToken): result of boolean, string
```

The success value is not interpreted. `Cmd.StartBackground(Id, Work)` starts an independent
one-shot operation after `Update` returns. Multiple one-shot operations may use the same id; the id
correlates failures. `Cmd.ReplaceSubscription(Id, Work)` gives a long-lived source a unique positive
id. Replacement requests cancellation immediately and returns control to the host without waiting
for the old source. The host joins it once complete, then starts the latest pending replacement.
Further replacements for that id overwrite the pending work; they do not start intermediate
operations or overlap the old source. Input, updates, and paint continue while cancellation is pending.
`Cmd.CancelSubscription(Id)` requests cancellation, clears any pending replacement, and is a no-op
when that id is not active. Each application may own at most 256 simultaneous operations, including
sources awaiting cancellation. Each source has at most one pending replacement.

Completed successful operations are reaped silently. A returned `Error`, panic, or runtime error is
delivered on the main application thread as:

```pascal
TuiMsg.BackgroundFailed(Id, Kind, Message, Code, Line, Column)
```

`Kind` is a `TaskFailureKind`. Cancellation caused by replacement, explicit subscription
cancellation, or shutdown is not reported as a failure. Other failures discovered while replacing
or cancelling are delivered before application messages from the replacement are processed. If a
failure is discovered only during final shutdown, the host releases the terminal first and then raises that failure because no
further `Update` call exists. Shutdown discards pending replacements and joins existing work.

Background work owns no TUI state. Only `Update`, `UpdateApplication`, command application, `View`,
layout, and paint may change the application and all run serially on the host task. Each channel
preserves FIFO order. Pending routed framework messages are drained first. When both Console and
application channels remain ready, the interactive host alternates their selection priority so
neither source permanently starves the other. There is no total ordering between independent
sources.

The interactive Console reader runs as host-owned work and polls native input with a 50 ms timeout.
It yields its pool worker after every poll, including polls without input.
While selecting input, the root host does not execute queued background computations inline;
those remain on the VM worker pool.
Application messages wake the selection immediately; a 50 ms host timer probes task groups so even
a worker that panics before sending a message becomes `BackgroundFailed`. Shutdown
cancels and joins that reader, then cancels and joins application work, releases the terminal, and
closes the application inbox. Cancellation is cooperative; work that neither checks its token nor
uses cancellable waits can delay shutdown.

## Headless execution

`TuiApplication.OpenForTest(Size)` opens a fixed-size host.
`OpenForTestWithPalette(Size, Palette)` supplies a custom initial palette, and
`App.Palette()` returns the current value for assertions.
`RunIterations` renders an initial frame before consuming its iteration budget;
one processed message consumes one iteration. `SurfaceSnapshot` is the explicit
copying boundary for assertions, including semantic cell roles.

The host owns the working surface and clears pending work when `Close()` is
called.

For a typed inbox, `RunBackgroundIterations` uses the same two update callbacks and delivers
`TuiMsg.Started` once after its first frame. A framework message already queued by routing or task
failure wins the next iteration; otherwise one available application message is consumed; otherwise
the iteration receives `TuiMsg.Tick`. `InjectBackgroundForTest(Inbox, Message)` delegates to
`TrySend`, preserving the same FIFO, queue-full, and closed results as production. Call
`CloseWithBackground(Inbox)` to cancel and join owned work before closing the inbox. A later send is
rejected.

## Interactive terminal

`TuiApplication.Run(InitialModel, Update, View)` owns one process terminal for
its duration. It uses the same initial-render and update ordering as the
headless driver, presents completed frames through `Std.Console`, and blocks
for terminal input when no routed messages remain. Idle applications do not
receive implicit ticks and do not rebuild or repaint their view.

`TuiMsg.Tick` remains available for explicitly injected messages and for
deterministic time steps produced by `RunIterations`. The interactive host does
not assume that every application needs an animation timer.

Keyboard, mouse, and positive resize events are normalized before routing;
paste and focus events are ignored. Mouse coordinates become zero-based.
Consecutive native resize events are coalesced to the latest dimensions before
layout and painting, so keyboard input following a resize burst is not delayed
by intermediate frames that do not reflect application state.
`Std.Console` owns raw mode, alternate-screen, input features, and cursor
rollback for the interactive session.

`TuiApplication.RunWithPalette(InitialModel, Update, View, Palette)` has the
same lifecycle with a caller-defined initial palette.

The first terminal frame transfers the complete logical surface. Later frames
compare glyphs, semantic style roles, concrete cell styles, and wide-glyph
continuation positions with the last presented surface. Adjacent changes are
transferred as damage runs; an unchanged frame performs no Console frame write.

## Headless frame and routing order

Every processed message follows:

```text
dequeue → Update → command check → View → validate → layout → paint
```

Injected events are FIFO. Pending routed messages are drained before another
external input is read. Unsupported Console events and empty routing results
return directly to the blocking input wait without calling `Update`, `View`, or
paint. A `Quit` command stops before another `View` or paint.

Tab moves through the active focusable subtree except when a `TextArea` is
focused, where it inserts two spaces. Character and editing keys produce
controlled `TextChanged` or `TextAreaChanged` messages. Enter or Space activates
a focused enabled button; otherwise Enter activates the enabled default button.
Alt plus a button mnemonic activates its enabled button inside the active modal
or full tree. When an enabled button owns focus, Up/Left moves to the previous
enabled button and Down/Right moves to the next one. Navigation wraps and stays
inside the active modal subtree. Escape produces `QuitRequested`.

Focused one-line inputs support Home/End, Ctrl+Left/Ctrl+Right word movement,
and Ctrl+Backspace/Ctrl+Delete word removal. Up/Down produces ordinary
`TuiMsg.Key` for a plain input; an input built with `MakeHistoryInput` instead
uses those keys to traverse its explicit history and restore its draft.

Left-button pointer downs hit-test the previous arranged frame. Ordinary
buttons capture the press, paint selected while the pointer remains inside, and
emit their action only on a left-button release inside the same button.
Dragging outside cancels the selected appearance and releasing outside does not
activate the button. Focus changes precede the pointer message that repaints
the pressed state. Other controls retain their immediate controlled changes,
and unhandled pointer input remains `TuiMsg.Pointer`. A press inside a one-line
input focuses it and proposes the caret nearest the clicked visible cell.
Panels and overlays paint an opaque normal surface with a thin rounded frame.
An overlay is fixed and centered; it has no drag state or shadow.
Menu bars, menu popups, and status lines likewise paint their complete chrome
with dedicated menu or status roles. Mnemonics and status key hints retain
their shortcut role inside normal or selected chrome. Menu popups are flat
framed overlays without shadows.
An open hierarchical menu handles Escape before the application-level quit
request. Menu shortcuts, F10, mnemonics, arrows, and popup pointer hits are
described in [Menus](menus.md).

`InjectResizeForTest` replaces the host surface size before `TuiMsg.Resize`
reaches `Update`. When an overlay is present directly under the desktop, key
and pointer targeting is limited to the last overlay subtree.

Painting reads the arranged-frame index only. The private clipped canvas uses
local coordinates and nested origins/clips. Overwriting either half of a wide
glyph clears both columns.

## See also

- [`Std.Tui`](README.md)
- [Mandelbrot typed-inbox and replaceable-subscription example](../../../../examples/math/mandelbrot/README.md)
- [Elements](elements.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Testing](../testing/README.md)
