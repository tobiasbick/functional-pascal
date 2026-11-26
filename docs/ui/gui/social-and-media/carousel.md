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
public function View(Slides: array of E.Element<int>; Selected: int): E.Element<int>;
begin
  return W.Carousel<int>('release-tour', Slides, Selected,
    function(Index: int): int => Index, new W.Options(Height := 200.0F, AutoAdvance := 6.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Height / Gap` | Both zero. Zero uses native sizing and spacing. |
| `AutoAdvance / Loop` | 0 / false. Positive seconds enable automatic advance. Loop wraps manual navigation at the ends. |
| `HideArrows / HideDots / Enabled` | false / false / true. |

## Interaction

Use arrows, dots or a horizontal gesture to select another slide. Store
the proposed index to accept manual or timed movement.

## See also

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
