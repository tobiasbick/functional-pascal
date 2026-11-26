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
| `Name / Path` | Supply the display name and a model-owned image path. Null displays initials. |
| `OnBrowse` | null. Supply a message factory to handle browse activation. |
| `Size / Enabled` | 88 logical points / true. Zero size selects the native default. |

## Interaction

CUI proposes a dropped image after decoding it. Store the proposed path to
accept the new photo. Use `ImageExtensions` for the file chooser to match
the drop filter. The chooser returns a path, so validate the file when
handling the selection.

## See also

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
