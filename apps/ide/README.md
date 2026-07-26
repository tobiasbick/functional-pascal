# FPAS IDE

The IDE is a supported single-document terminal application on `Std.Tui`. It
edits one UTF-8 `.fpas` source, can retain one `.fpasprj` project session or
one `.fpasworkspace` with all of its direct member projects, and provides Open,
Save, Check, Run, and Exit.

The menu bar contains File, Edit, Run, and Options. Options > Theme switches
immediately between Classic Blue, Dark, and Monochrome. The selected theme is
kept in the application model for the current run; it is not persisted.
Classic Blue uses a Turbo Vision-inspired blue surface with cyan frames and
selection plus yellow titles and shortcuts. Dark keeps those cyan and yellow
accents on black and dark-gray surfaces. Monochrome remains black and white.

## Run

Run it from the repository root:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj
```

Optionally open one UTF-8 `.fpas` file, `.fpasprj` project, or
`.fpasworkspace` at startup:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj -- example.fpas
```

Only one optional path is accepted.

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

When a menu is open, use arrows, Enter, Space, Escape, or the highlighted
mnemonic. Pointer presses open popups and submenus or activate their commands.
Pointer movement over an open menu switches root popups, selects commands, and
opens nested submenus without activating them.

Open, Save As, and dirty-document dialogs use Turbo Vision-inspired frames with
right/bottom shadows. Drag a dialog title bar with the left mouse button to
move it within the visible desktop. The next dialog opens centered again.

The editor supports character insertion, Enter, Backspace, Delete, arrows,
Home, End, Page Up, Page Down, and two-space Tab insertion. Its status line
shows dirty state and the one-based caret line and column.

Click inside Messages to focus it and scroll captured output. When focus is
already outside the editor, Tab can also reach it. Up, Down, Page Up, Page Down,
Home, and End move its vertical viewport. New non-Check messages return the
viewport to their first line.

A failed Check selects the first diagnostic for the open source and reveals its
header in the viewport. The selected header begins with `>`. Scrolling past
another diagnostic selects it; press Enter while Messages is focused to move
the editor caret and viewport to that line and column. Diagnostics belonging to
other files are not navigable. Run output remains plain captured output.

## Files and processes

Open uses a path-entry dialog and accepts `.fpas`, `.fpasprj`, or
`.fpasworkspace`. Opening a project validates its manifest, retains its
original text and resolved direct source list, and loads the program main file
or the first library/test source.

Opening a workspace validates its manifest and every member project, retaining
the original workspace text, normalized paths, member order, and complete
direct project models. Its first member becomes the active project and supplies
the initial source document. Workspace data is internal: no workspace tree,
project selector, or dependency view is displayed.

Opening a standalone source clears project and workspace state. Opening a
project directly clears workspace state. Project and workspace opening is
atomic, so a failed manifest, member, or source read leaves the current session
unchanged.

Save writes the active source path; saving an untitled document opens Save As.
Failed reads and writes leave the document unchanged. Open and Exit protect
modified text with Save/Discard/Cancel.

Check and Run save the document and synchronously invoke the same `fpas`
executable that launched the IDE. A standalone session uses the source path; a
project or workspace session uses the active project's retained manifest path
so its complete direct source set participates. The message area shows the exit
code followed by normalized stdout and stderr. A failed save prevents the
process from starting. Child programs must be non-interactive because their
output is captured and no stdin is connected.

The message panel retains the latest complete result and shows three lines
inside its bounded scroll viewport. Diagnostic markers are display-only and do
not modify the captured report. Resize the terminal when the terminal-too-small
overlay is shown.

## Development checks

Run the headless IDE tests with:

```text
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
```
