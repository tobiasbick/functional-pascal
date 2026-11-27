# Presence pill

Use a presence pill to summarize collaborators. The gallery shows three
avatars and highlights the speaking participant.
Available in **GUI**.

![Presence pill and speaking indicator in the FPAS gallery](images/presence-pill.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.PresencePill;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Canvas.LiveCursors as L;
uses Fpas.Ui.Widgets.Canvas.PresencePills as W;

// Shows the current collaborators.
// ## Parameters
// - `People`: Collaborator names and presence state.
// ## Returns
// The native presence indicator.
public function View(People: array of L.Person): E.Element of string;
begin
  return W.PresencePill of string('presence', People);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `People` | Reuse the collaborator data from the [cursor overlay](live-cursors.md). |
| `Person.Avatar` | Empty. Supply a PNG or JPEG path for a picture. The pill uses initials if the path is empty or the picture fails to load. |
| `Person.Away` | `false`. Set to `true` to dim the avatar. |
| `Person.Speaking` | `false`. The first speaking person who is present gets level bars and a name. A visible avatar also gets a ring. |
| `Size` | `0` uses a diameter of 26 logical units. |
| `MaxVisible` | `0` shows up to five avatars. The pill counts the remaining people in a `+n` marker. |

## Interaction

Update the collaborator array as people join, leave or speak. Hover over an
avatar to see the person's name and away status. The pill takes no keyboard
focus and emits no message. With reduced motion, the level bars stay still.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_live_cursors)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Canvas/PresencePills.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Live cursors](live-cursors.md)
- [Canvas and bars](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [PresencePill.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/PresencePill.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.PresencePill.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.PresencePill.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiWindow.PresencePill.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.PresencePill.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [presence_pill.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/presence_pill.c)
  - [presence_pill.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/presence_pill.h)
