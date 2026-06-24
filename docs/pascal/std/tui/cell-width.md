# Terminal cell width

Unicode display-width policy for hosted TUI painting and layout.

## Policy

Functional Pascal TUI uses **Unicode display width** (via the `unicode-width` crate) to map
scalars to terminal columns:

| Width | Behavior |
| --- | --- |
| `0` | Combining marks and other zero-width characters do not advance layout on their own. |
| `1` | ASCII, box-drawing, and most symbols occupy one column. |
| `2` | East Asian wide characters occupy two columns; the second column is filled with a space continuation cell when painting. |

Ambiguous-width characters use neutral width. Truncation reserves the last visible column for
`…` when more text remains.

## Where it applies

- Console `write_text_at` / `write_char_at` (`crates/fpas-std/src/console/screen/text_at.rs`)
- Shared helpers in `crates/fpas-std/src/text/cell_width.rs`
- Frame title slots, menu bar/popup geometry, status bar segments, and basic controls (labels,
  buttons, list box, input line cursor placement)

## Implementation (contributors)

| Layer | Location |
| --- | --- |
| Cell-width helpers | `crates/fpas-std/src/text/cell_width.rs` |
| Console paint | `crates/fpas-std/src/console/screen/text_at.rs` |
| Control truncation | `crates/fpas-std/src/tui/widget/control/mod.rs` |
| FPAS workflow regression | [`tests/tui/tui_cell_width_test.fpas`](../../../../tests/tui/tui_cell_width_test.fpas) |

## See also

- [TUI index](README.md)
- [Hosted dispatch](app/README.md)
- [Frames](app/frames.md)
