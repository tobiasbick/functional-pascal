# IDE product target

The first supported IDE is a compact, keyboard-friendly full-screen application
for one Functional Pascal source file. Its visual identity is a simple blue
character-cell workbench rather than a desktop of independent windows.

## Fixed screen

```text
┌ Open ─ Save ─ Check ─ Run ─ Exit ─────────────────────────────────┐
│ /absolute/or/relative/path/to/program.fpas                         │
│ ┌ Editor ───────────────────────────────────────────────────────┐ │
│ │ program Example;                                             │ │
│ │                                                              │ │
│ │ begin                                                        │ │
│ │   ...                                                        │ │
│ │ end.                                                         │ │
│ └──────────────────────────────────────────────────────────────┘ │
│ ┌ Messages ─────────────────────────────────────────────────────┐ │
│ │ Check succeeded                                              │ │
│ └──────────────────────────────────────────────────────────────┘ │
└ modified | Ln 4, Col 3 | F2 Save | F9 Check | Ctrl+F9 Run ──────┘
```

The command bar, message area, and status line have fixed roles. The editor gets
all remaining space. The message area has a small fixed height and clips or
scrolls long output; it is not a second managed pane.

## Commands

| Command | Required behavior |
| --- | --- |
| Open | Show a path-entry dialog; protect dirty content before replacing it. |
| Save | Write the current UTF-8 text; ask for a path when the document is untitled. |
| Check | Save, run `fpas check <file>`, and show exit code/stdout/stderr. |
| Run | Save, run `fpas run <file>`, and show exit code/stdout/stderr. |
| Exit | Quit immediately when clean; otherwise show Save/Discard/Cancel. |

The command bar is flat. It does not require nested menus. The minimum keyboard
bindings are F2 Save, F9 Check, Ctrl+F9 Run, and Alt+X Exit. Pointer activation
uses normal `Std.Tui` hit testing.

## Startup

- With no program argument, open an empty untitled document.
- With one `.fpas` argument, read that file and place the caret at the start.
- Reject extra arguments and non-`.fpas` paths with a visible message instead of
  a panic.
- `.fpasprj` and `.fpasworkspace` are not startup targets in this plan.

## Editing behavior

The editor supports insertion, Enter, Backspace, Delete, arrows, Home, End,
PageUp, PageDown, and Tab insertion. The model owns text, caret, and scroll
offset. Caret movement keeps the caret visible. The first version has no
selection, clipboard, undo/redo, search, syntax coloring, or automatic indent.

All text is UTF-8. Caret indexing follows the existing `Std.Str` indexing
contract used by `Std.Tui.Input`.

## Process behavior

Check and Run are synchronous in the first version. The IDE remains blocked
until the child process exits. Interactive child stdin, cancellation, background
jobs, and terminal handoff are outside this plan. Captured stdout and stderr are
displayed separately and remain available until the next command.

## Explicit non-goals

- Help screens or contextual help.
- Project/workspace parsing and a source tree.
- More than one document.
- Tiled, split, floating, movable, or resizable panes.
- A generic desktop/window manager.
- Language services, debugger integration, completion, or syntax highlighting.
- Preserving APIs or behavior from the current unsupported `apps/ide` sources.
