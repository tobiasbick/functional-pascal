# IDE project and workspace tree

**Status:** Complete on 2026-07-15. Session state and tree window are implemented.

## Goal

When the FPAS IDE opens a `.fpasprj` project or `.fpasworkspace` workspace, it automatically adds a non-modal Turbo Vision window containing an expandable tree:

```text
workspace.fpasworkspace
└─ application.fpasprj
   └─ src
      ├─ main.fpas
      └─ ui
         └─ menu.fpas
```

The tree is display-only in this scope. It opens with the project or workspace and remains a desktop window until the application exits. Selecting a file, opening an editor, refresh, filtering, and arbitrary filesystem browsing are explicitly out of scope.

Project and workspace manifests remain TOML. Do not migrate them to JSON.

## Dependency order

All steps complete:

1. `Std.Toml` — complete
2. `Std.Fs.Glob` — complete
3. IDE session state — complete
4. IDE project/workspace tree window — complete

## Handoff

Implemented behavior is documented in [`docs/pascal/apps/ide.md`](../pascal/apps/ide.md).

IDE tests under `apps/ide/tests/workspace/` and `apps/ide/tests/shell/open_tree_test.fpas` cover source resolution, tree construction, and the non-modal window after `File / Open`.

Future IDE work (file activation, editor, refresh) is not tracked in this file.
