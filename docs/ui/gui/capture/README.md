# Capture

The Capture page accepts files from the desktop, replaces a profile photo
and records a handwritten signature. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Dropzone](dropzone.md) | Accept one or more files by drag and drop or open a file chooser. |
| [Avatar upload](avatar-upload.md) | Replace a profile photo by dropping or selecting an image. |
| [Signature pad](signature-pad.md) | Draw strokes with a pointing device and read their coordinates. |

Return a `FileDialogs.OpenFile` command from the application update to handle
browse requests. Use the same extension filter for the drop target and the
chooser. The application validates file content and size before using a path.

## See also

- [UI handbook](../README.md)
- [Capture in the gallery](../../../../examples/ui/gui/gallery/Pages/Capture/View.fpas)
- [Gallery file handling](../../../../examples/ui/gui/gallery/Pages/Capture/Update.fpas)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Capture/State.fpas)
- [CaptureContract.fpas](../../../../tests/Fixtures/gui-host/CaptureContract.fpas)
- [CuiCaptureTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/CuiCaptureTests.cs)
- [WindowsCaptureInput.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/WindowsCaptureInput.cs)
