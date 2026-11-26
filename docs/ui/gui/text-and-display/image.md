# Image

The gallery displays a rounded illustration from in-memory pixels.
Available in **GUI**.

![Rounded landscape illustration in the FPAS gallery](images/image.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Image;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Media.Images as W;
uses Fpas.Ui.Widgets.Media.MediaImages as Media;

// Displays a rounded picture.
// ## Parameters
// - `Picture`: File or tightly packed straight-alpha RGBA pixels.
// ## Returns
// The image at the gallery's logical size.
public function View(Picture: Media.Source): E.Element of string;
begin
  return W.Image of string('preview', Picture, new W.Options(
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
array when replacing the entire image.

For a changed rectangle, keep the returned source in your model. For example,
`MediaImages.Update(Picture, 10, 20, 2, 3, Patch, 12)` replaces two pixels in
each of three rows. Each row contributes eight RGBA bytes. The next source
row starts twelve bytes later. Omit the stride for tightly packed input.
Coordinates must fit the image, and the operation copies the supplied bytes.

The result contains a complete immutable CPU snapshot. A widget displaying
the preceding revision updates just that rectangle on its existing texture.
A new widget, or one that skipped a revision, uploads the complete snapshot.
The same source works in images, image viewers and custom canvas painting.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_image)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Media/Images.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Display/View.fpas)
- [Media sources](../../../../src/Fpas.Ui/Widgets/Media/MediaImages.fpas)
- [Image viewer](../social-and-media/image-viewer.md)
- [Chapter index](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Media/`
  - [Image.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Media/Image.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Media/`
  - [NativeMediaSource.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Media/NativeMediaSource.cs)
  - [NativeMethods.Image.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Media/NativeMethods.Image.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Media/`
  - [CuiMediaImage.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiMediaImage.Types.cs)
  - [CuiMediaImage.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiMediaImage.cs)
  - [CuiWindow.Image.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiWindow.Image.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/media/`
  - [image.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/image.c)
  - [image.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/image.h)
  - [media_image.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/media_image.c)
  - [media_image.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/media_image.h)
- [Software and OpenGL image update tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/CuiImageUpdateTests.cs)
