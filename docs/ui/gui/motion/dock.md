# Dock

The gallery places its application icons in a centered dock.
Available in **GUI**.

![App dock with running dots and a mail badge in the FPAS gallery](images/dock.png)

## Example

```pascal
unit Docs.Ui.Motion.Dock;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Navigation.Docks as W;
uses System as Sys;

// Shows a small application dock.
// ## Parameters
// - `FilesRunning`: Whether to show the running dot on Files.
// ## Returns
// The dock with a message when Files is opened.
public function View(FilesRunning: bool): E.Element of string;
begin
  return W.Dock of string('dock', [
    (W.Entry of string).Item(Sys.Char.ConvertFromUtf32($E0D7), 'Files', 'open-files',
      C.Color.Automatic, FilesRunning, 0)
  ]);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Entry.Item / Separator` | Pass an icon, label, activation message and appearance. Separator divides groups. |
| `Running / Badge` | Running draws a dot. A positive Badge shows a count. |
| `Size / Magnify / Enabled` | 0 / 0 / true. Zero uses native size or magnification. |

## Interaction

Move the pointer across icons to magnify them. Handle the item message
to activate its destination and update running indicators or badges.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Docks.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/Controls.fpas)
- [Tab bar](tab-bar.md)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [Dock.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/Dock.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.Dock.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.Dock.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.Dock.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.Dock.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [dock.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/dock.c)
  - [dock.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/dock.h)
