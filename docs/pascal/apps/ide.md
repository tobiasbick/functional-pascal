# FPAS IDE

The FPAS IDE is a supported single-document terminal application built on
`Std.Tui`. It can also retain one directly opened FPAS project or one workspace
with its direct member projects. Its fixed screen contains:

- a File/Edit/Run/Options menu bar with popups and submenus;
- an optional source path;
- a controlled multiline editor;
- a message area;
- a dirty/caret status line.

Run the current application from the repository root:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj
```

Pass at most one `.fpas`, `.fpasprj`, or `.fpasworkspace` path after `--`. A
source path loads one clean UTF-8 document. A project path establishes a
project session and loads its program main file or the first direct
library/test source. A workspace path retains all member projects in declared
order and opens the initial source of its first member. Loading errors leave
the previous session unchanged; startup errors retain an empty untitled
document.

## Controls

| Input | Action |
| --- | --- |
| F2 | Save |
| F9 | Check |
| Ctrl+F9 | Run |
| Alt+X | Exit |
| F10 | Open and focus the menu |
| Alt+F / Alt+E / Alt+R / Alt+O | Open a top-level menu |
| Tab | Insert two spaces in the editor; move focus elsewhere |
| Enter or Space | Activate the focused command or dialog button |
| Enter in Messages | Jump to the selected Check diagnostic |
| Escape | Cancel a dialog, or request Exit |
| Left pointer button | Focus and activate commands or dialog buttons |

Options > Theme switches immediately between the bundled Classic Blue, Dark,
and Monochrome palettes. The selected theme is retained for the current run
but is not persisted. Classic Blue uses a Turbo Vision-inspired blue surface
with cyan frames and selection plus yellow titles and shortcuts. Dark retains
the cyan and yellow hierarchy on black and dark-gray surfaces. Monochrome uses
only black, white, and gray. Open menus support arrows, Enter, Space, Escape,
mnemonics, and pointer activation. Pointer movement over an open menu switches
root popups, selects commands, and opens nested submenus without activating
them.

The editor handles character insertion, Enter, Backspace, Delete, arrows, Home,
End, Page Up, Page Down, and two-space Tab insertion. Caret movement updates the
viewport to keep the caret visible.

The Messages window is also focusable. Click inside it, or reach it with Tab
when focus is already outside the editor. Then use Up, Down, Page Up, Page Down,
Home, or End to scroll its captured output. A new status or non-Check process
report resets the message viewport to its first line.

Check diagnostics for the open source are navigable. The first diagnostic is
selected automatically and revealed after Check. Its header is prefixed with
`>`; scrolling past another diagnostic selects that location. Enter while
Messages is focused moves the editor caret and viewport to the selected
one-based line and column. Stale locations are clamped to the current text.
Diagnostics for other files and Run output remain non-navigable.

## Document lifecycle

Open accepts a `.fpas`, `.fpasprj`, or `.fpasworkspace` path in a modal dialog.
Save writes the current UTF-8 text to its known source path; an untitled
document uses the Save As path dialog. A failed read or write updates the
message area without changing the current document.

Open, Save As, and dirty-document dialogs use Turbo Vision-inspired frames with
right/bottom shadows. Drag a dialog title bar with the left mouse button to
move it within the visible desktop. Closing a dialog resets its position, so
the next dialog opens centered.

The IDE parses project TOML itself. It validates the project name and kind,
program main file, and source include/exclude arrays. Patterns are resolved
relative to the manifest directory. The model retains the original manifest
text, normalized manifest/root paths, typed project kind, optional main file,
and a stable deduplicated list of direct `.fpas` sources. Dependency projects
are not flattened. Opening a standalone source clears this project state.

Project opening is atomic: both the manifest and initial source must load
successfully before the current session changes. The fixed path row shows the
project name with the active source. The retained source list is internal data
for a later project tree; no tree is displayed yet.

The IDE also parses workspace TOML itself. It retains the normalized workspace
path and root, original manifest text, workspace name, and all validated member
projects in manifest order. Duplicate member paths and duplicate
case-insensitive project names are rejected. The first member is the active
project and supplies the initial document. Loading is atomic across the
workspace, every member manifest, and that document.

Workspace state is retained only for later UI work. The current screen does not
display a workspace tree, project selector, dependency view, or member list.
Opening a standalone source clears project and workspace state; directly
opening a project clears workspace state.

Modified text is derived from the difference between the editor text and its
last successfully loaded or saved value. Open and Exit require a
Save/Discard/Cancel decision before continuing.

## Check and Run

Check and Run first save the current text. After a successful save, they invoke
the current `fpas` executable synchronously as `check <path>` or `run <path>`.
The path is the source in a standalone session and the active project manifest
in a project or workspace session. The message area retains a deterministic
report containing the exit code, stdout, and stderr; empty streams are shown as
`(empty)`. A failed save aborts the command without starting a process.

The IDE is blocked while Check or Run executes. Run is intended for
non-interactive programs because child output is captured and child stdin is
not connected. The message panel retains the latest complete result and shows
three lines inside its bounded scroll viewport. Its selection marker does not
change the retained process report.

When the terminal is too small for the fixed screen, the application displays
the terminal-too-small overlay until it is resized.

See [`apps/ide/README.md`](../../../apps/ide/README.md) for development and test
commands.
