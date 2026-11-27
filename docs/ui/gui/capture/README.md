# Capture

The Capture page accepts files from the desktop, replaces a profile photo
and records a handwritten signature. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Dropzone](dropzone.md) | Accept one or more files by drag and drop or open a file chooser. |
| [Avatar upload](avatar-upload.md) | Replace a profile photo by dropping or selecting an image. |
| [Signature pad](signature-pad.md) | Draw strokes with a pointing device and read their coordinates. |

## Capture gallery page

The tall dropzone displays the first accepted path below two compact targets.
The compact targets sit in fill-width boxes inside a wrapping row. They show
the native drop feedback and leave the displayed path unchanged.

The 88-unit avatar and its caption share a centered row. Dropping an image
replaces the initials or the previous photo. A 520-unit box holds the
180-unit signature pad and its native Clear button.

The gallery follows CUI's drag-and-drop example. To handle browse activation
in an application, return a `FileDialogs.OpenFile` command from the update.
Use the same extension filter for the drop target and the chooser. Validate
file content and size before using a path.

## See also

- [UI handbook](../README.md)
- [Capture in the gallery](../../../../examples/ui/gui/gallery/Pages/Capture/View.fpas)
- [Gallery file handling](../../../../examples/ui/gui/gallery/Pages/Capture/Update.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#capture-evidence)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Capture/State.fpas)
- [CaptureContract.fpas](../../../../tests/Fixtures/gui-host/CaptureContract.fpas)
- [CuiCaptureTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/CuiCaptureTests.cs)
- [WindowsCaptureInput.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/WindowsCaptureInput.cs)
