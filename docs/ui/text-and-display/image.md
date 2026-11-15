# Image

The gallery displays a rounded illustration from in-memory pixels.
Available in **GUI**.

![Rounded landscape illustration in the FPAS gallery](images/image.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Image;

uses
  Fpas.Ui.Elements as E, Fpas.Ui.Layout as L,
  Fpas.Ui.Widgets.Media.Images as W, Fpas.Ui.Widgets.Media.MediaImages as Media;

// Displays a rounded picture.
// ## Parameters
// - `Picture`: File or tightly packed straight-alpha RGBA pixels.
// ## Returns
// The image at the gallery's logical size.
public function View(Picture: Media.Source): E.Element<string>;
begin
  return W.Image<string>('preview', Picture, new W.Options(
    Width := L.Extent.Fixed(L.Measure.Logical(234)),
    Height := L.Extent.Fixed(L.Measure.Logical(150)),
    Radius := 10.0F
  ));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width / Height` | 64 / 64 logical points. |
| `Radius / Tint` | 0 / null. Tint multiplies the source colors. |

## Usage

For generated pixels, use `MediaImages.Source.Rgba` with four bytes per
pixel. Keep the array unchanged while the view uses it and pass a new
array when the image changes.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Media/Images.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Display/View.fpas)
- [Media sources](../../../src/Fpas.Ui/Widgets/Media/MediaImages.fpas)
- [Image viewer](../social-and-media/image-viewer.md)
- [Chapter index](README.md)
