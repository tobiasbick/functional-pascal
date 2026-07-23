# FPAS IDE replacement

Status: not started. The next task is [Phase 1 — captured process execution](implementation-phases.md#phase-1--captured-process-execution).

This plan replaces every source file currently under `apps/ide/src/` with a
small, supported IDE built on the current `Std.Tui` Model-Update-View API. The
first product is a fixed full-screen, single-document Pascal IDE. It is not a
general windowing environment.

## Fixed decisions

- The application remains at `apps/ide/` and keeps the project names `ide` and
  `ide-core`.
- Existing `apps/ide/src/` code is deleted when the replacement application
  phase begins. It is not an API or architecture baseline.
- The first release edits one `.fpas` file at a time.
- The screen has one command bar, one editor, one message area, and one status
  line in a fixed layout.
- `Std.Tui` remains an immutable element tree with model-owned state.
- A generic controlled multiline `TextArea` is added to `Std.Tui`; the standard
  library does not gain an IDE-specific widget.
- Check and Run invoke the current `fpas` executable and display captured output.
- File dialogs are path-entry dialogs. No filesystem browser is required.
- There is no Help command or help subsystem.
- Tiling, split panes, movable windows, overlapping windows, tabs, project trees,
  plugins, syntax highlighting, completion, debugging, and multiple open files
  are outside this plan.
- There is no backward-compatibility requirement for the existing IDE source.

## Definition of success

The replacement is complete when:

1. `fpas run apps/ide/ide.fpasprj -- [optional-file.fpas]` opens the IDE.
2. A file can be opened, edited across multiple lines, saved, checked, and run.
3. Compiler stdout, stderr, and exit status appear in the message area.
4. Unsaved changes are protected before Open and Exit.
5. Keyboard and pointer input work through the current `Std.Tui` routing model.
6. Headless tests cover the editor control and the IDE's important model slices.
7. The old IDE modules and exports no longer exist.
8. Current documentation describes the supported IDE, and this future plan is
   removed from `docs/future/`.

## Documents

| Document | Purpose |
| --- | --- |
| [Product](product.md) | Visible screen, commands, workflows, and non-goals. |
| [Architecture](architecture.md) | State ownership, module layout, and required APIs. |
| [Replacement inventory](replacement-inventory.md) | Exact disposition of current IDE files. |
| [Implementation phases](implementation-phases.md) | Ordered work, tests, verification, and completion log. |

## Restart procedure after context loss

1. Read `AGENTS.md` and every document in `docs/future/ide/`.
2. Run `git status --short`; preserve unrelated changes and do not clean the
   worktree.
3. Read the status line at the top of this file and the matching phase section.
4. Inspect the current files named by that phase. Do not assume this plan's
   inventory is newer than the worktree.
5. Run the phase's baseline command before editing when practical.
6. Implement only that phase and its explicitly listed prerequisites.
7. Update the phase checkbox, the status line above, and the completion log in
   `implementation-phases.md` in the same change.
8. Do not commit unless the user requests a commit.

If code and plan disagree, treat implemented code plus passing tests as evidence,
then correct the plan before continuing. Do not silently reinterpret a phase.
