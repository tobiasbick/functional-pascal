# Dot-matrix display

Use a dot-matrix display for text in CUI's 5×7 font. The gallery reports
whether the bedtime estimate is above or below the threshold. Available in **GUI**.

![Dot-matrix display in the FPAS gallery](images/dot-display.png)

## Example

```pascal
unit Docs.Ui.Instruments.DotDisplay;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Display.DotDisplays as W;

// Displays the status at bedtime.
// ## Parameters
// - `AboveLimit`: Whether the estimate exceeds the threshold.
// ## Returns
// The display for the application view.
public function View(AboveLimit: bool): E.Element of string;
begin
  const Status: string := if AboveLimit then 'OVER' else 'UNDER' end if;
  return W.DotDisplay of string('bed-display', Status + ' THE LINE AT BEDTIME',
    new W.Options(Pitch := 3.0F, Accent := AboveLimit));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Pitch` | `0` uses the native spacing of 4 logical points between dot centers. CUI clamps positive values to 1.5–40 logical points. |
| `Color` | `Automatic` uses the theme foreground for lit dots. Use `Theme` or `Rgba` to set the color. |
| `Accent` | `false`. If enabled, the theme accent takes precedence over `Color`. |
| `Dim` | `true`. Draws dim dots for unlit positions, including the space between glyphs. |
| `Marquee` | `false`. Scrolls a line wider than its layout width when motion is enabled. |
| `Align` | `Start`. `Start` and `End` follow the reading direction. `Left` and `Right` use physical edges. `Center` centers the line. |
| `Width` | `Content`. Use `Fill`, `Fixed` or `Percent` to constrain the line. |

## Interaction

The display is passive. Change the text in the view when the model changes.
The glyphs run from left to right. CUI draws printable ASCII, degree signs and
middle dots. CUI uses the middle-dot glyph for bullets. Other characters
appear as boxes.

If `Marquee` is enabled and the host allows motion, a line wider than the
display scrolls at full pitch. Static displays and displays with reduced
motion shrink the dots to fit.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_dot_display)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Display/DotDisplays.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Instruments/View.fpas)
- [Instruments](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Display/`
  - [DotDisplay.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Display/DotDisplay.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Display/`
  - [NativeMethods.DotDisplay.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Display/NativeMethods.DotDisplay.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Display/`
  - [CuiWindow.DotDisplay.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Display/CuiWindow.DotDisplay.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/display/`
  - [dot_display.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/dot_display.c)
  - [dot_display.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/dot_display.h)
