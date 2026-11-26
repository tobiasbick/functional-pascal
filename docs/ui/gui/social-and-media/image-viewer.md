# Image viewer

The gallery shows a landscape with native zoom controls and pointer-centered zoom.
Available in **GUI**.

![Landscape and image toolbar in the FPAS gallery](images/image-viewer.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.ImageViewer;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Media.ImageViewers as W;
uses Fpas.Ui.Widgets.Media.MediaImages as M;

// Shows an image at the current zoom and center.
// ## Parameters
// - `Picture`: Local file or immutable RGBA image.
// - `Current`: Model-owned zoom and center.
// ## Returns
// The viewer with proposed pan and zoom as messages.
public function View(Picture: M.Source; Current: W.Value): E.Element of W.Value;
begin
  return W.ImageViewer of W.Value('landscape', Picture, Current,
    function(Change: W.Value): W.Value => Change,
    new W.Options(Height := L.Extent.Fixed(L.Measure.Logical(260))));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Zoom / CenterX / CenterY` | All zero. Zero Zoom fits the image. Positive Zoom gives logical points per image pixel. Centers are image coordinates. |
| `Width / Height / MaximumZoom` | Content / Content / 0. Zero MaximumZoom uses the native maximum. |
| `HideToolbar / Checkerboard / Enabled` | false / false / true. |

## Interaction

Use the wheel to zoom around the pointer and drag to pan. Double-click
switches from fit or a zoom below 100 percent to 100 percent; otherwise it
fits the image. Store the proposed value. To request fit from the
application, set Zoom to zero.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_image_viewer)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Media/ImageViewers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/Media.fpas)
- [Image cropper](image-cropper.md)
- [Social and media](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Media/`
  - [ImageViewer.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Media/ImageViewer.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Media/`
  - [NativeMethods.ImageViewer.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Media/NativeMethods.ImageViewer.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Media/`
  - [CuiWindow.ImageViewer.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiWindow.ImageViewer.Types.cs)
  - [CuiWindow.ImageViewer.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiWindow.ImageViewer.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/media/`
  - [image_viewer.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/image_viewer.c)
  - [image_viewer.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/image_viewer.h)
