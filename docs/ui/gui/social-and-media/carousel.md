# Carousel

Use a carousel for a sequence of views. The gallery presents four release
slides with arrows, dots and optional timed advance.
Available in **GUI**.

![Release carousel in the FPAS gallery](images/carousel.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.Carousel;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Media.Carousels as W;

// Shows the current release-tour slide.
// ## Parameters
// - `Slides`: Composed slides in display order.
// - `Selected`: Current zero-based index.
// ## Returns
// The carousel with a proposed slide index.
public function View(Slides: array of E.Element of int; Selected: int): E.Element of int;
begin
  return W.Carousel of int('release-tour', Slides, Selected,
    function(Index: int): int => Index, new W.Options(Height := 200.0F, AutoAdvance := 6.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Height / Gap` | Both zero. CUI uses 220 logical units for the slide height and 12 between slides. |
| `AutoAdvance` | Zero disables timed movement. Use a positive interval in seconds to advance and wrap through the slides. |
| `Loop` | `false`. Set `true` to wrap manual navigation at the ends. |
| `HideArrows / HideDots / Enabled` | false / false / true. |

## Interaction

Supply one to 64 slides in display order. Drag or use the horizontal
wheel over a slide. Use Left and Right while the viewport has focus.
The arrow buttons and dots are available unless you hide them.

Store the proposed zero-based index to accept manual or timed movement.
Timed movement pauses while the pointer rests over the carousel
viewport. [CUI-075](../../../future/cui-upstream-bugs.md#cui-075)

The carousel clips slides to the viewport. If the enclosing layout
changes its width, the host requests a frame to fit each slide.
[CUI-074](../../../future/cui-upstream-bugs.md#cui-074)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_carousel)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Media/Carousels.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/Media.fpas)
- [Social and media](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Media/`
  - [Carousel.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Media/Carousel.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Media/`
  - [NativeMethods.Carousel.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Media/NativeMethods.Carousel.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Media/`
  - [CuiWindow.Carousel.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiWindow.Carousel.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/media/`
  - [carousel.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/carousel.c)
  - [carousel.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/carousel.h)
