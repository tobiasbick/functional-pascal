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

## Headless execution

`TuiApplication.OpenForTest(Size)` opens a fixed-size host.
`RunIterations` renders an initial frame before consuming its iteration budget;
one processed message consumes one iteration. `SurfaceSnapshot` is the explicit
copying boundary for assertions, including semantic cell roles.

The host owns the working surface and clears pending work when `Close()` is
called.

## Interactive terminal

`TuiApplication.Run(InitialModel, Update, View)` owns one process terminal for
its duration. It uses the same initial-render and update ordering as the
headless driver, presents completed frames through `Std.Console`, and waits up
to 16 ms for an event before emitting `Tick(16)`.

Keyboard, mouse, and positive resize events are normalized before routing;
paste and focus events are ignored. Mouse coordinates become zero-based.
Consecutive native resize events are coalesced to the latest dimensions before
layout and painting, so keyboard input following a resize burst is not delayed
by obsolete intermediate frames.
`Std.Console` owns raw mode, alternate-screen, input features, and cursor
rollback for the interactive session.

The first terminal frame transfers the complete logical surface. Later frames
compare glyphs, semantic style roles, and wide-glyph continuation positions
with the last presented surface. Adjacent changes are transferred as damage
runs; an unchanged frame performs no Console frame write.

## Headless frame and routing order

Every processed message follows:

```text
dequeue → Update → command check → View → validate → layout → paint
```

Injected events are FIFO. Pending routed messages are drained before another
external input is read. Empty routing results do not synthesize a tick. A
`Quit` command stops before another `View` or paint.

Tab moves through the active focusable subtree except when a `TextArea` is
focused, where it inserts two spaces. Character and editing keys produce
controlled `TextChanged` or `TextAreaChanged` messages; Enter or Space activates
a focused button. Escape produces `QuitRequested`. Left-button pointer downs
hit-test the previous arranged frame: focus changes are queued before an action
or controlled change, and unhandled pointer input remains `TuiMsg.Pointer`.

`InjectResizeForTest` replaces the host surface size before `TuiMsg.Resize`
reaches `Update`. When a dialog is present directly under the desktop, key and
pointer targeting is limited to the last such dialog subtree.

Painting reads the arranged-frame index only. The private clipped canvas uses
local coordinates and nested origins/clips. Overwriting either half of a wide
glyph clears both columns.

## See also

- [`Std.Tui`](README.md)
- [Elements](elements.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Testing](../testing/README.md)
