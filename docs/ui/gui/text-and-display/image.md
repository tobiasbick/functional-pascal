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
| `Width`, `Height` | 64 logical units each. `Content` also uses 64. |
| `Radius` | 0 logical units. |
| `Tint` | `Charts.Color.Automatic` preserves the source colors. `Charts.Color.Theme(Token)` or `Charts.Color.Rgba(Value)` multiplies them by the selected color. The renderer resolves theme tokens for each frame. RGBA with zero alpha uses the automatic tint. |

## Usage

Use `MediaImages.Source.File(Path)` for a PNG or JPEG file. The GUI host
decodes the file with CUI and caches the texture by source path. An image
with an unreadable file shows no picture.

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

The GUI host owns the image texture. It creates the texture with
`cui_image_create`. It derives the native ID with `cui_make_id` from the
widget's stable ID in the current CUI scope. The image comparison uses a
separate ID for each image. The host keeps each texture in `cui_state`
and registers it for cleanup, like `cui_state_on_free`. Cleanup calls
`cui_image_destroy` when the texture is replaced or retired.

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
- [Native color parity test](../../../../src/Fpas.Ui.Gui.Cui/native/tests/display.c)
