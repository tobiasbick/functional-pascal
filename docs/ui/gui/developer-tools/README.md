# Developer tools

The gallery inspects traces, network requests and project state. It also
demonstrates editors for filters, schedules and keyboard bindings. These
widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Trace viewer](trace-viewer.md) | Inspect spans across lanes on a shared time axis. |
| [Network waterfall](network-waterfall.md) | Compare the timing phases of HTTP requests. |
| [Dependency graph](dependency-graph.md) | Show a directed graph of packages, build targets or services. |
| [Commit graph](commit-graph.md) | Show commits, branches, merges and refs in history order. |
| [Pipeline](pipeline.md) | Summarize a CI run by stage and job. |
| [Problems panel](problems-panel.md) | Filter diagnostic messages and open their source locations. |
| [Key-value editor](key-value-editor.md) | Edit headers, query parameters or other named values. |
| [Query builder](query-builder.md) | Build nested groups of filter conditions. |
| [Schedule picker](schedule-picker.md) | Edit a cron expression and preview upcoming runs. |
| [Keymap](keymap.md) | Display shortcuts, categories and conflicting bindings. |

## Developer tools gallery page

The page shows the ten widgets above in table order. It prepares trace spans
with the native sort and displays the native description of the current query.
See the [CUI gallery comparison](../../../future/cui-compare/README.md#developer-tools-evidence).

## See also

- [UI handbook](../README.md)
- [More developer tools](../more-developer-tools/README.md)
- [Developer tools in the gallery](../../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)

## Source files

<details>
<summary>Browse 12 source files</summary>

- `examples/ui/gui/gallery/Pages/Developer/`
  - [Samples.fpas](../../../../examples/ui/gui/gallery/Pages/Developer/Samples.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Developer/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Developer/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeDeveloperData.Inspectors.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeDeveloperData.Inspectors.cs)
  - [NativeDeveloperEditors.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeDeveloperEditors.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [DeveloperData.Editors.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/DeveloperData.Editors.Types.cs)
  - [DeveloperData.Editors.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/DeveloperData.Editors.cs)
  - [DeveloperData.Inspectors.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/DeveloperData.Inspectors.Types.cs)
  - [DeveloperData.Inspectors.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/DeveloperData.Inspectors.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [developer_inspectors.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/developer_inspectors.c)
  - [developer_tools.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/developer_tools.c)
- `tests/Fixtures/gui-host/`
  - [DeveloperToolsContract.fpas](../../../../tests/Fixtures/gui-host/DeveloperToolsContract.fpas)

</details>
