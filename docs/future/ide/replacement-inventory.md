# IDE replacement inventory

This inventory is a deletion map, not a reuse promise. Inspect the worktree
before acting because files may have changed after this plan was written.

## Stable paths

| Path | Treatment |
| --- | --- |
| `apps/ide/` | Keep as the application root. |
| `apps/ide/ide.fpasworkspace` | Rewrite members only if the final project set changes. |
| `apps/ide/ide.fpasprj` | Keep name and program role; rewrite source list and dependencies. |
| `apps/ide/ide-core.fpasprj` | Keep name and library role; replace all exports. |
| `apps/ide/README.md` | Replace status-only text with supported run/use instructions at completion. |

## Source deletion map

Delete every current file below when Phase 3 starts:

| Current area | Files | Replacement |
| --- | --- | --- |
| Shell | `src/shell/shell.fpas` | `src/app/update.fpas`, `src/app/view.fpas` |
| UI chrome | `src/ui/menu.fpas`, `status.fpas`, `theme.fpas` | `src/app/actions.fpas`, `src/app/view.fpas` |
| Dialogs | `src/dialog/dialog.fpas`, `open.fpas`, `about.fpas` | modal state and view in `src/app/`; no About dialog |
| Workspace state | `src/workspace/model.fpas`, `session.fpas` | single-document model in `src/document/model.fpas` |
| Manifest loading | `src/workspace/load.fpas`, `sources.fpas`, `classify.fpas` | none; projects/workspaces are outside scope |
| Tree UI | `src/workspace/tree_build.fpas`, `tree_window.fpas` | none; no project tree |
| Entry point | `src/main.fpas` | new argument parser and MVU bootstrap at the same path |

Do not retain aliases, compatibility units, deprecated exports, or commented-out
copies of deleted code. Git history is the recovery mechanism.

## Manifest end state

`ide-core.fpasprj` exports only the new units:

```text
Ide.App.Model
Ide.App.Actions
Ide.App.Update
Ide.App.View
Ide.Document.Model
Ide.Document.Io
Ide.Process.Runner
```

`ide.fpasprj` remains a program with `main = "src/main.fpas"` and a workspace
dependency on `ide-core`. Its source include remains limited to the entry point.

## Repository references to update

When the replacement becomes runnable, update these in the same phase:

- `apps/ide/README.md`
- `docs/pascal/apps/ide.md`
- `docs/pascal/apps/README.md`
- `examples/README.md`
- example-check allowlists in `crates/fpas-cli/src/main_tests/examples.rs`

Search for obsolete unit names with:

```text
rg -n "Ide\.(Shell|Ui|Dialog|Workspace)" apps docs examples crates tests
```

Completion requires zero matches except an explicit migration assertion in a
test, if one is genuinely useful.
