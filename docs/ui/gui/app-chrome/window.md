# Window state and placement

App chrome maximizes, minimizes, restores and moves the gallery window, and
reads its placement. An application saves the placement when it closes and
reopens the window there. Available in **GUI**.

![Window actions in the FPAS gallery](images/window-actions.png)

## Example

```pascal
unit Docs.Ui.AppChrome.Window;

uses Fpas.Ui.Gui.Cui as Cui, Fpas.Ui.Gui.Windows as Windows, Fpas.Ui.Mvu.Application as Mvu;

// Window actions of a document application.
public type Msg = variant
  // Maximize the window.
  Maximize;
  // Undo a maximize or a minimize.
  Restore;
  // Read the placement before closing.
  Save;
  // Keep the placement for the next start.
  Saved(Value: Cui.CuiWindowPlacement);
end variant;

// Returns the window command for an action and keeps the last placement.
// ## Parameters
// - `Kept`: Placement read last, or null before the first read.
// - `Message`: The window action.
// ## Returns
// The kept placement and the host command.
public function Update(
  Kept: Cui.CuiWindowPlacement?; Message: Msg
): Mvu.Transition<Cui.CuiWindowPlacement?, Msg>;
begin
  case Message of
    when Msg.Maximize:
      return new Mvu.Transition<Cui.CuiWindowPlacement?, Msg>(Kept, [Windows.Maximize<Msg>()]);
    when Msg.Restore:
      return new Mvu.Transition<Cui.CuiWindowPlacement?, Msg>(Kept, [Windows.Restore<Msg>()]);
    when Msg.Save:
      return new Mvu.Transition<Cui.CuiWindowPlacement?, Msg>(Kept, [Windows.ReadPlacement<Msg>(Msg.Saved)]);
    when Msg.Saved(const Value: Cui.CuiWindowPlacement):
      return new Mvu.Transition<Cui.CuiWindowPlacement?, Msg>(Value, []);
  end case;
end function;

end unit;
```

Pass the saved placement as `Options.Placement`, for example
`new Gui.Options(Title := 'Notes', Placement := Saved)`. The window opens with
the saved size and place, and maximized if it was. Closing the window does not
pass through the update. To save at the end, drive the session with `Step`
and read `Session.Placement()` after the last step and before `Dispose`.

## Commands

| Command in `Fpas.Ui.Gui.Windows` | Effect |
| --- | --- |
| `Maximize<TMsg>()` | Maximizes the window on its display. |
| `Minimize<TMsg>()` | Minimizes it to the taskbar or dock. |
| `Restore<TMsg>()` | Undoes a maximize or a minimize. A window minimized from maximized comes back maximized. |
| `MoveTo<TMsg>(X, Y)` | Places the top-left corner of the content. |
| `ReadPlacement<TMsg>(OnRead)` | Sends the window's `Cui.CuiWindowPlacement` to the update. |

## Placement

| Field | Meaning |
| --- | --- |
| `Width / Height` | Normal size in logical points, also while maximized or minimized. |
| `X / Y` | Top-left corner of the content in desktop coordinates: pixels on Windows and X11, points on macOS. |
| `HasPosition` | false where the system reports no place (Wayland). The system then places the window. |
| `State` | Normal or Maximized. A minimized window reports the state it comes back in. |

## Platform behavior

Keep the position only to pass it back. A place on a monitor that is no longer
there is left to the system, and the size is clamped to CUI's minimum window
size. On X11 the window manager carries out state changes in its own time, so
a new state can take a few frames to show. A window manager without EWMH
support ignores them. Wayland lets a window neither read nor choose its place.

## See also

- [Window commands](../../../../src/Fpas.Ui.Gui/Windows.fpas)
- [Host reference](../host.md#window-state-and-placement)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Desktop/HostActions.fpas)
- [Chapter index](README.md)
