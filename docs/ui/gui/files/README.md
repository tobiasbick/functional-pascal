# Files

Use a file browser as part of a view and a file picker for a modal choice.
The gallery also shows a project tree, a terminal with built-in demo commands
and a standalone path bar. The file picker is available in both hosts.

| Widget | When to use it |
| --- | --- |
| [File browser](file-browser.md) | Browse a folder and open a file in the application. |
| [File picker](file-picker.md) | Ask for a file to open, a save destination or a folder. |
| [Tree table](../data-grid/tree-table.md) | Inspect and rearrange a sample project hierarchy. |
| [Path bar](path-bar.md) | Navigate by segment, child menu or submitted path. |
| [Terminal](terminal.md) | Display ANSI output and receive command-line input. |

The browser and picker can both list local files or use a custom listing
provider. The screenshots use the gallery with a temporary sample workspace.

## Files gallery page

The [Files view](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)
places the browser and picker buttons above a project tree and terminal.
The project tree uses the native table, row menu and move operation. Its
changes apply to the sample hierarchy. The browser and terminal's `ls`
command share the same folder. A standalone path bar follows the original
sections.

## See also

- [UI handbook](../README.md)
- [Terminal Files chapter](../../tui/files/README.md)
- [Capture](../capture/README.md)
- [Files in the gallery](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)

## Source files

- [Folders.fpas](../../../../examples/ui/gui/gallery/Pages/Files/Folders.fpas)
- [ProjectTree.fpas](../../../../examples/ui/gui/gallery/Pages/Files/ProjectTree.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Files/State.fpas)
- [CuiFileSource.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiFileSource.cs)
