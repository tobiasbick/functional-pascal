# Window state and placement

App chrome maximizes, minimizes, restores and moves the gallery window, and
reads its placement. An application saves the placement when it closes and
reopens the window there. Available in **GUI**.

![Window actions in the FPAS gallery](images/window-actions.png)

## Example

```pascal
unit Docs.Ui.AppChrome.Window;

uses Fpas.Ui.Gui.Cui as Cui;
uses Fpas.Ui.Gui.Windows as Windows;
uses Fpas.Ui.Mvu.Application as Mvu;

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
): Mvu.Transition of (Cui.CuiWindowPlacement?, Msg);
begin
  case Message of
    when Msg.Maximize:
      return new Mvu.Transition of (Cui.CuiWindowPlacement?, Msg)(Kept, [Windows.Maximize of Msg()]);
    when Msg.Restore:
      return new Mvu.Transition of (Cui.CuiWindowPlacement?, Msg)(Kept, [Windows.Restore of Msg()]);
    when Msg.Save:
      return new Mvu.Transition of (Cui.CuiWindowPlacement?, Msg)(Kept, [Windows.ReadPlacement of Msg(Msg.Saved)]);
    when Msg.Saved(const Value: Cui.CuiWindowPlacement):
      return new Mvu.Transition of (Cui.CuiWindowPlacement?, Msg)(Value, []);
  end case;
end function;

end unit;
```

Pass the saved placement to `Desktop.Options.Placement`. The window opens
with the saved size, place and maximized state. Drive the session with
`Step` to save the final placement. Read `Session.Placement()` after
`Step` returns `false` and before `Dispose`.

Supply `OnCloseRequested` to receive a message when the user asks to close
the window. [Native close decisions](../host.md#application-lifecycle)

## Commands

| Command in `Fpas.Ui.Gui.Windows` | Effect |
| --- | --- |
| `SetTitle of TMsg(Title)` | Sets the title shown by the system. |
| `Resize of TMsg(Width, Height)` | Requests positive dimensions in logical units. |
| `Maximize of TMsg()` | Maximizes the window on its display. |
| `Minimize of TMsg()` | Minimizes it to the taskbar or dock. |
| `Restore of TMsg()` | Undoes a maximize or a minimize. A window minimized from maximized comes back maximized. |
| `MoveTo of TMsg(X, Y)` | Places the top-left corner of the content. |
| `ReadPlacement of TMsg(OnRead)` | Sends the window's `Cui.CuiWindowPlacement` to the update. |
| `ReadInfo of TMsg(OnRead)` | Sends the current `Windows.Info` to the update. |

## Placement

| Field | Meaning |
| --- | --- |
| `Width / Height` | Normal size in logical points, also while maximized or minimized. |
| `X / Y` | Top-left corner of the content in desktop coordinates: pixels on Windows and X11, points on macOS. |
| `HasPosition` | false where the system reports no place (Wayland). The system then places the window. |
| `State` | Normal or Maximized. A minimized window reports the state it comes back in. |

Use `Session.Placement()` for the normal size and the state to restore.

## Window observation

Use `Windows.ReadInfo` from `Update` to read `Windows.Info`.
A manually driven session can call `Session.InspectWindow()` on the window owner.

| Field | Meaning |
| --- | --- |
| `Size / State` | Logical dimensions and current native state, including minimized. |
| `Backend / NativeWindow` | Platform name and borrowed platform handle. |
| `DpiScale` | Drawable pixels per logical unit. Updated when CUI refreshes the viewport. |
| `Focused` | Whether the window has native focus. |
| `FileDragActive` | Whether the platform reports files being dragged over the window. |
| `DroppedFileCount` | Number of files left unclaimed in the most recently presented frame. |

CUI reports drag hover where the platform supplies it. A dropzone that
claims a drop receives its paths and leaves the window count at zero.
Direct `CuiWindow` drawing can read `DroppedFileCount` while constructing
a frame. Read it after the dropzones to give them first access.
Between frames it reads the last presented count. Each presented frame
updates that count.

## Platform behavior

Keep the position only to pass it back. A place on a monitor that is no longer
there is left to the system, and the size is clamped to CUI's minimum window
size. A fixed-size window ignores maximize requests.
On X11 the window manager carries out state changes in its own time, so
a new state can take a few frames to show. A window manager without EWMH
support ignores them. Wayland lets a window neither read nor choose its place.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Window commands](../../../../src/Fpas.Ui.Gui/Windows.fpas)
- [Host reference](../host.md#window-state-and-placement)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Desktop/HostActions.fpas)
- [Chapter index](README.md)

## Source files

- [Window observation tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiWindowObservationTests.cs)

- `src/Fpas.Ui.Gui.Cui/Hosting/`
  - [CuiWindow.Placement.cs](../../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Placement.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Hosting/`
  - [NativeMethods.Window.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Window.cs)
- `src/Fpas.Ui.Gui.Cui/native/host/`
  - [window_state.c](../../../../src/Fpas.Ui.Gui.Cui/native/host/window_state.c)
- `tests/Fpas.Ui.Gui.Cui.Tests/Windowing/`
  - [CuiPlacementTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiPlacementTests.cs)
  - [CuiPresentationTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiPresentationTests.cs)
  - [CuiWindowTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiWindowTests.cs)
