# FPAS IDE

Turbo Vision desktop application under [`apps/ide/`](../../../apps/ide/). The IDE shell provides menu and status chrome; workspace state tracks the opened project or workspace root.

## Workspace modules

```text
apps/ide/src/workspace/
 ├── classify.fpas    path → OpenKind
 ├── model.fpas       LoadedProject, LoadedWorkspace, SessionRoot
 ├── load.fpas        manifest parsing (Std.Fs + Std.Toml)
 ├── sources.fpas     `[sources]` include/exclude → relative `.fpas` paths
 ├── tree_build.fpas  source paths → OutlineNode tree
 ├── tree_window.fpas non-modal desktop tree window
 └── session.fpas     in-memory root state and open files
```

## File / Open

`File / Open` classifies the chosen path:

| Extension | Behavior |
| --- | --- |
| `.fpasworkspace` | Load workspace manifest, replace session root, show tree window titled with workspace name |
| `.fpasprj` | Load project manifest, replace session root, show tree window titled with project name |
| `.fpas` / `*_test.fpas` | Register as an open source file; tree unchanged |

Manifest loading uses [`Std.Toml`](../std/text/toml.md). Source patterns expand through [`Std.Fs.Glob`](../std/host/fs.md).

## Project tree data

For a project root the IDE:

1. Expands every `[sources].include` entry relative to the project directory.
2. Removes paths matched by `[sources].exclude`.
3. Retains `project.main` even when an exclude pattern would remove it.
4. Groups relative paths into directory nodes (directories before files, sorted).

## Tree window

After a successful project or workspace open:

- A non-modal `Window` is added with `Desktop.Add` (not `ExecView` / modal `Dialog`).
- Content is an `Outline.New` tree. Workspace, project, and directory nodes start expanded; file nodes are leaves.
- Opening another root reuses the same window (`Window.SetTitle` + `Outline.SetNodes`).

Display-only in the current scope: no file activation, editor, refresh, or arbitrary filesystem browsing.

## See also

- [Projects](../program-structure/projects.md) — manifest fields used by the loader
- [Workspaces](../program-structure/workspaces.md)
- [`Std.Tui` application facade](../std/tui/app/README.md)
- Plan archive: [`docs/future/ide-project-tree.md`](../../future/ide-project-tree.md)
