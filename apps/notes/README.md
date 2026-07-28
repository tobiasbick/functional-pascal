# Notes

Notes is a modern local-first terminal note-taking application built entirely
with Functional Pascal and [`Std.Tui`](../../docs/pascal/std/tui/README.md).
It demonstrates a complete multi-unit application rather than a focused syntax
example.

## Run

Use `notes/` in the current directory:

```sh
fpas run apps/notes/notes.fpasprj
```

Open another directory by passing it after `--`:

```sh
fpas run apps/notes/notes.fpasprj -- ./my-notes
```

The directory is created when its parent exists. Notes never scans outside the
selected directory.

## Interface

Wide terminals show a searchable note list beside the editor:

```text
┌ NOTES · 12 active · Ctrl+K commands ────────────────────────────────────────┐
│ ┌ Notes ────────────────────┐ ┌ Editor ★ ─────────────────────────────────┐ │
│ │ Search notes…             │ │ Thoughts about terminal applications      │ │
│ │ ★ TUI design · fpas, tui  │ │ fpas, tui                                 │ │
│ │   Release checklist       │ │ ── Markdown ───────────────────────────── │ │
│ │   Parser ideas            │ │ A focused application should feel like…  │ │
│ └───────────────────────────┘ └────────────────────────────────────────────┘ │
│ Ready  Ctrl+N new  Ctrl+S save  Ctrl+F search  Ctrl+1/2 panes  F1 help     │
└─────────────────────────────────────────────────────────────────────────────┘
```

In the wide layout, the note list uses one third of the terminal width, clamped
to 32 through 52 cells. Longer titles and tags are shortened with an ellipsis
without splitting Unicode grapheme clusters, so note contents never move the
divider. Below 65 columns, the list and editor become separate full-width
panes. `Ctrl+1` focuses note search, `Tab` moves into the note list, and
selecting another note keeps focus in that list. `Ctrl+2` switches to the
editor; `Enter` on a selected list item opens it directly in the Markdown
editor. Below 50 by 14 cells, Notes displays a size requirement instead of
drawing a broken layout.

The custom truecolor palette uses a dark blue-gray background, cyan focus and
shortcut accents, amber warnings, and coral failures. Modal overlays provide
the command palette, keyboard help, save errors, and the unsaved-change exit
flow.

## Keyboard

| Key | Action |
| --- | --- |
| `Ctrl+N` | Create a note |
| `Ctrl+S` | Save the selected note |
| `Ctrl+F` | Focus search |
| `Ctrl+P` | Toggle the selected note's pin |
| `Ctrl+K` | Open the command palette |
| `Ctrl+1` | Focus note search |
| `Ctrl+2` | Focus the editor |
| `Enter` | Open the selected list item in the Markdown editor |
| `Alt+X` | Quit, confirming first when changes are unsaved |
| `F1` | Open keyboard help |
| Arrow keys | Move between command and dialog buttons |
| `Esc` | Close an overlay; otherwise do nothing |

Mouse selection, input placement, buttons, terminal resize, and text-area
scrolling use the normal `Std.Tui` routing behavior.

## `.note` format

Every note is one UTF-8 file named `<id>.note`. The stable file name does not
change when the title changes. TOML frontmatter stores metadata; everything
after the closing delimiter is the Markdown body. Notes writes canonical LF
line endings and accepts LF or CRLF when loading externally edited files.

```text
+++
format = 1
id = "1785266230123-481927"
title = "Thoughts about terminal applications"
created_ms = 1785266230123
updated_ms = 1785267418450
tags = ["fpas", "tui", "design"]
pinned = true
archived = false
+++
The body remains ordinary Markdown text.
```

Unknown or malformed files are not silently discarded. Notes keeps them as
load issues and reports their count in the status text. Unsupported format
versions return an explicit parse error.

Notes uses `Std.Fs.WriteTextAtomic` when creating or replacing files. The
current note is saved with `Ctrl+S`, before switching notes, when pinning or
archiving, and through the save-and-quit action. Editing remains in memory and
does not create periodic TUI ticks or write once per keystroke.

Archiving sets `archived = true`; version 1 does not permanently delete files.

## Project layout

```text
apps/notes/
├── notes-core.fpasprj       # library exporting Notes.* units
├── notes.fpasprj            # runnable program
└── src/
    ├── notes.fpas           # arguments, loading, terminal host
    └── Notes/
        ├── Model.fpas       # persisted and MVU state
        ├── Format.fpas      # .note parser and encoder
        ├── Repository.fpas  # directory loading and durable saves
        ├── Theme.fpas       # truecolor palette
        ├── Update.fpas      # state transitions and shortcuts
        └── View.fpas        # responsive TUI and overlays
```

Regression tests live under [`tests/apps/notes/`](../../tests/apps/notes/) and
use the same `notes-core` library as the application.
