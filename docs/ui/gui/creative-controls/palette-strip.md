# Palette strip

The gallery varies a five-swatch palette with a model-owned shuffle seed.
Available in **GUI**.

![Five swatches and a shuffle button in the FPAS gallery](images/palette-strip.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.PaletteStrip;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Pickers.PaletteStrips as W;

// Displays the current palette.
// ## Parameters
// - `Current`: Swatches, locks and optional shuffle seed.
// ## Returns
// The palette with lock and shuffle proposals.
public function View(Current: W.Value): E.Element of W.Event;
begin
  return W.PaletteStrip of W.Event('palette', Current,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Field | Default or requirement | Behavior |
| --- | --- | --- |
| `Value.Label` | Required | Sets the caption. Begin with `##` to hide the caption. |
| `Value.Swatches` | One to sixteen swatches | Sets the colors and locks in display order. |
| `Swatch.Color` | Required | Sets the swatch color. |
| `Swatch.Locked` | `false` | Marks the swatch with a lock and preserves its color during shuffle. |
| `Value.Seed` | `null` | Gives CUI a shuffle seed. If `null`, the application chooses how to shuffle. |
| `Options.Display` | Empty | Sets the caption opposite the label. |
| `Options.NoShuffle` | `false` | Hides the Shuffle button when `true`. |
| `Options.Height` | `0` | Sets the logical swatch height. Zero uses the native height of 32. |
| `Options.Plain` | `false` | Removes the surrounding well when `true`. |
| `Options.Enabled` | `true` | Accepts pointer and keyboard input when `true`. |
| `Options.NoLocks` | `false` | Hides swatch locks and includes every swatch in shuffle when `true`. |

## Interaction

The swatch under the pointer or keyboard focus widens and shows its hex code
when space permits. Press Tab to reach the swatches and the Shuffle button.
Left and Right move between swatches in the direction of the arrow.
To toggle a lock, click the swatch or press Enter or Space while it has focus.

| Result | Behavior |
| --- | --- |
| `Event.Swatches` | Contains the proposed swatch colors and locks. |
| `Event.Seed` | Contains the proposed seed. It is `null` if the application omits `Value.Seed`. |
| `Event.Changed` | Is `true` after a lock toggle or a seeded shuffle. |
| `Event.Shuffled` | Is `true` when the Shuffle button activates. |

With a seed, CUI recolors the unlocked swatches and advances the seed.
The seed advances even if every swatch is locked. Store the proposed swatches
and seed in the model to continue the same shuffle sequence. With a null seed,
use `Shuffled` to choose the new colors in the application.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_palette_strip)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/PaletteStrips.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [PaletteStrip.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/PaletteStrip.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.PaletteStrip.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.PaletteStrip.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.PaletteStrip.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.PaletteStrip.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [palette_strip.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/palette_strip.c)
  - [palette_strip.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/palette_strip.h)
