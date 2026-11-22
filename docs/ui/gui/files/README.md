# Files

Use a file browser as part of a view and a file picker for a modal choice.
The gallery adds a standalone path bar and a terminal with built-in demo
commands. The file picker is available in both hosts. This chapter covers the GUI.

| Widget | When to use it |
| --- | --- |
| [File browser](file-browser.md) | Browse a folder and open a file in the application. |
| [File picker](file-picker.md) | Ask for a file to open, a save destination or a folder. |
| [Path bar](path-bar.md) | Navigate by segment, child menu or submitted path. |
| [Terminal](terminal.md) | Display ANSI output and receive command-line input. |

The browser and picker can both list local files or use a custom listing
provider. The screenshots use the gallery with a temporary sample workspace.

## See also

- [UI handbook](../README.md)
- [Terminal Files chapter](../../tui/files/README.md)
- [Capture](../capture/README.md)
- [Files in the gallery](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)

## Source files

- [Folders.fpas](../../../../examples/ui/gui/gallery/Pages/Files/Folders.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Files/State.fpas)
- [CuiFileSource.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiFileSource.cs)
