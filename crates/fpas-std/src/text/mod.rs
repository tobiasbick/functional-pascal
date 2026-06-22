//! Shared Unicode terminal cell-width helpers for console and TUI widgets.
//!
//! **Documentation:** `docs/pascal/std/tui/cell-width.md`

mod cell_width;

pub(crate) use cell_width::{
    WIDE_CONTINUATION, char_display_offset, display_width, layout_display_cells, str_display_width,
    text_cells_for_paint, truncate_for_title_slot,
};
