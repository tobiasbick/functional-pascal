# FPAS IDE

The FPAS IDE is a supported single-document terminal application built on
`Std.Tui`. Its fixed screen contains:

- a flat Open/Save/Check/Run/Exit command bar;
- an optional source path;
- a controlled multiline editor;
- a message area;
- a dirty/caret status line.

Run the current application from the repository root:

```text
cargo run -q -p fpas-cli -- run --std-lib lib apps/ide/ide.fpasprj
```

Pass at most one `.fpas` path after `--`. The IDE reads that UTF-8 file into a
clean document and reports an error while retaining an empty untitled document
when loading fails. `.fpasprj` and `.fpasworkspace` files are not IDE startup
targets.

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

Open accepts a path in a modal dialog. Save writes the current UTF-8 text to its
known path; an untitled document uses the Save As path dialog. A failed read or
write updates the message area without changing the current document.

Modified text is derived from the difference between the editor text and its
last successfully loaded or saved value. Open and Exit require a
Save/Discard/Cancel decision before continuing.

## Check and Run

Check and Run first save the current text. After a successful save, they invoke
the current `fpas` executable synchronously as `check <path>` or `run <path>`.
The message area retains a deterministic report containing the exit code,
stdout, and stderr; empty streams are shown as `(empty)`. A failed save aborts
the command without starting a process.

The IDE is blocked while Check or Run executes. Run is intended for
non-interactive programs because child output is captured and child stdin is
not connected. The message panel retains the latest complete result and shows
three lines inside its bounded scroll viewport. Its selection marker does not
change the retained process report.

When the terminal is too small for the fixed screen, the application displays
the terminal-too-small overlay until it is resized.

See [`apps/ide/README.md`](../../../apps/ide/README.md) for development and test
commands.
