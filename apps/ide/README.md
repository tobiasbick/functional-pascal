# FPAS IDE

The IDE is a supported single-document terminal application on `Std.Tui`. It
edits one UTF-8 `.fpas` source and provides Open, Save, Check, Run, and Exit.

## Run

Run it from the repository root:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj
```

Optionally open one UTF-8 `.fpas` file at startup:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj -- example.fpas
```

Only one optional `.fpas` path is accepted. Project and workspace manifests are
not IDE startup targets.

## Controls

| Input | Action |
| --- | --- |
| F2 | Save |
| F9 | Check |
| Ctrl+F9 | Run |
| Alt+X | Exit |
| Tab | Insert two spaces in the editor; move focus elsewhere |
| Enter or Space | Activate the focused command or dialog button |
| Enter in Messages | Jump to the selected Check diagnostic |
| Escape | Cancel a dialog, or request Exit |
| Left pointer button | Focus and activate commands or dialog buttons |

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

Open uses a path-entry dialog. Save writes to the current path; saving an
untitled document opens Save As. Failed reads and writes leave the document
unchanged. Open and Exit protect modified text with Save/Discard/Cancel.

Check and Run save the document and synchronously invoke the same `fpas`
executable that launched the IDE with `check <path>` or `run <path>`. The
message area shows the exit code followed by normalized stdout and stderr. A
failed save prevents the process from starting. Child programs must be
non-interactive because their output is captured and no stdin is connected.

The message panel retains the latest complete result and shows three lines
inside its bounded scroll viewport. Diagnostic markers are display-only and do
not modify the captured report. Resize the terminal when the terminal-too-small
overlay is shown.

## Development checks

Run the headless IDE tests with:

```text
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
```
