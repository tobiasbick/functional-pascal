# Avatar upload

Use an avatar upload to replace a profile photo by dropping an image on the
avatar or opening the native chooser. The gallery shows initials before a
photo is selected. Available in **GUI**.

![Avatar upload in the FPAS gallery](images/avatar-upload.png)

## Example

```pascal
unit Docs.Ui.Capture.AvatarUpload;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Gui.FileDialogs as Dialogs;
uses Fpas.Ui.Mvu.Application as Mvu;
uses Fpas.Ui.Widgets.Files.AvatarUploads as W;

// Requests a new profile photo.
public type Msg = variant
  // Open the chooser.
  Browse;
  // Adopt the proposed path.
  SetPhoto(Path: string);
end variant;

// Builds the editable profile photo.
// ## Parameters
// - `Path`: Current image path, or null to show initials.
// ## Returns
// The avatar upload for the application view.
public function View(Path: string?): E.Element of Msg;
begin
  return W.AvatarUpload of Msg('profile-photo', 'Maya Okafor', Path, Msg.SetPhoto,
    function(): Msg => Msg.Browse);
end function;

// Handles the Browse message.
// ## Returns
// The command to include in the update result.
public function OpenChooser(): Mvu.Command of Msg;
begin
  return Dialogs.OpenFile of Msg(Msg.SetPhoto, 'Select a profile photo', 'Images', W.ImageExtensions);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable widget identity. |
| `Name` | Display name for the initials and their background color. |
| `Path` | Model-owned image path. Null or an unavailable image displays initials. |
| `OnDrop` | Maps the path of a decoded drop to an application message. |
| `OnBrowse` | null. Supply a message factory to handle a click or press of Enter or Space while focused. |
| `Size` | 88 logical points. The diameter must be finite. Nonpositive values use 88. |
| `Enabled` | true. Controls focus, browse activation and file drops. |

## Interaction

CUI draws a pencil badge and shows a camera overlay while the pointer or a
file drag is over the visible avatar. CUI takes the first dropped path with a
`png`, `jpg`, `jpeg`, `bmp`, `gif` or `tga` extension and proposes it only
after decoding the image. A successful drop takes precedence over browse
activation in the same frame.

Store the proposed path to keep the new photo. FPAS owns the native texture
and draws the model's path again on the next frame if the drop is declined.
Use `ImageExtensions` to match the chooser's filter to the drop target.
The chooser returns a path. Apply any application-specific image size
requirements when handling either selection.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_avatar_upload)
- [Widget API](../../../../src/Fpas.Ui/Widgets/Files/AvatarUploads.fpas)
- [Avatar](../disclosure/avatar.md)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Capture/View.fpas)
- [Capture](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Files/`
  - [AvatarUpload.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Files/AvatarUpload.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Files/`
  - [NativeMethods.AvatarUpload.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Files/NativeMethods.AvatarUpload.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Files/`
  - [CuiWindow.AvatarUpload.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiWindow.AvatarUpload.Types.cs)
  - [CuiWindow.AvatarUpload.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiWindow.AvatarUpload.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/files/`
  - [avatar_upload.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/avatar_upload.c)
  - [avatar_upload.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/avatar_upload.h)
- [Native media tests](../../../../src/Fpas.Ui.Gui.Cui/native/tests/media.c)
