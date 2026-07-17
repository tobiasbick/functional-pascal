use std::collections::HashMap;

use super::cell::SavedRegionId;

pub(super) const DEFAULT_SCREEN_WIDTH: u16 = 80;
pub(super) const DEFAULT_SCREEN_HEIGHT: u16 = 25;
pub(super) const TEXT_MODE_BW40: i64 = 0;
pub(super) const TEXT_MODE_C40: i64 = 1;
pub(super) const TEXT_MODE_BW80: i64 = 2;
pub(super) const TEXT_MODE_C80: i64 = 3;
pub(super) const TEXT_MODE_CO40: i64 = 4;
pub(super) const TEXT_MODE_CO80: i64 = 5;
pub(super) const TEXT_MODE_MONO: i64 = 7;

mod cells;
mod color;
mod frames;
mod regions;
mod writing;

use color::RenderColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WindowRect {
    pub(super) left: u16,
    pub(super) top: u16,
    pub(super) right: u16,
    pub(super) bottom: u16,
}

impl WindowRect {
    pub(super) fn full(width: u16, height: u16) -> Self {
        Self {
            left: 1,
            top: 1,
            right: width,
            bottom: height,
        }
    }

    pub(super) fn width(self) -> u16 {
        self.right - self.left + 1
    }

    pub(super) fn height(self) -> u16 {
        self.bottom - self.top + 1
    }

    pub(super) fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameDamage {
    FullFrame,
    Rect(WindowRect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScreenCell {
    pub(super) glyph: String,
    pub(super) fg: RenderColor,
    pub(super) bg: RenderColor,
    pub(super) continuation: bool,
}

#[derive(Debug, Clone)]
struct SavedRegion {
    rect: WindowRect,
    cells: Vec<ScreenCell>,
}

#[derive(Debug, Clone)]
pub(super) struct ConsoleState {
    pub(super) width: u16,
    pub(super) height: u16,
    window: WindowRect,
    pub(super) cursor_x: u16,
    pub(super) cursor_y: u16,
    pub(super) fg: u8,
    pub(super) bg: u8,
    active_fg: RenderColor,
    active_bg: RenderColor,
    pub(super) cursor_visible: bool,
    pub(super) cursor_big: bool,
    pub(super) last_mode: i64,
    pub(super) crt_mode: bool,
    /// Deferred wrap: the cursor reached the last column but has not yet
    /// advanced to the next line. The next character write triggers the wrap.
    pending_wrap: bool,
    cells: Vec<ScreenCell>,
    /// Previous frame for differential rendering. Empty until the first render.
    prev_cells: Vec<ScreenCell>,
    /// Mutated screen region since the last committed present.
    pending_frame_damage: Option<FrameDamage>,
    saved_regions: HashMap<SavedRegionId, SavedRegion>,
    next_saved_region_id: u64,
}

impl ConsoleState {
    pub(super) fn new(width: u16, height: u16) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let blank = ScreenCell {
            glyph: " ".into(),
            fg: RenderColor::Crt(7),
            bg: RenderColor::Crt(0),
            continuation: false,
        };
        Self {
            width,
            height,
            window: WindowRect::full(width, height),
            cursor_x: 1,
            cursor_y: 1,
            fg: 7,
            bg: 0,
            active_fg: RenderColor::Crt(7),
            active_bg: RenderColor::Crt(0),
            cursor_visible: true,
            cursor_big: false,
            last_mode: TEXT_MODE_C80,
            crt_mode: false,
            pending_wrap: false,
            cells: vec![blank; width as usize * height as usize],
            prev_cells: Vec::new(),
            pending_frame_damage: None,
            saved_regions: HashMap::new(),
            next_saved_region_id: 1,
        }
    }

    pub(super) fn window_width(&self) -> u16 {
        self.window.width()
    }

    pub(super) fn window_height(&self) -> u16 {
        self.window.height()
    }

    pub(super) fn screen_width(&self) -> i64 {
        i64::from(self.width)
    }

    pub(super) fn screen_height(&self) -> i64 {
        i64::from(self.height)
    }

    pub(super) fn resize(&mut self, width: u16, height: u16) {
        let new_width = width.max(1);
        let new_height = height.max(1);
        if self.width == new_width && self.height == new_height {
            return;
        }

        let blank = ScreenCell {
            glyph: " ".into(),
            fg: self.active_fg,
            bg: self.active_bg,
            continuation: false,
        };
        let old_width = self.width;
        let old_height = self.height;
        let old_cells = self.cells.clone();
        let mut new_cells = vec![blank; new_width as usize * new_height as usize];

        for y in 1..=old_height.min(new_height) {
            for x in 1..=old_width.min(new_width) {
                let old_idx = ((y - 1) * old_width + (x - 1)) as usize;
                let new_idx = ((y - 1) * new_width + (x - 1)) as usize;
                new_cells[new_idx] = old_cells[old_idx].clone();
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.cells = new_cells;
        self.normalize_wide_cells();
        self.prev_cells.clear();
        self.pending_frame_damage = Some(FrameDamage::FullFrame);

        self.window.left = self.window.left.min(new_width);
        self.window.top = self.window.top.min(new_height);
        self.window.right = self.window.right.min(new_width).max(self.window.left);
        self.window.bottom = self.window.bottom.min(new_height).max(self.window.top);

        self.cursor_x = self.cursor_x.min(self.window.width());
        self.cursor_y = self.cursor_y.min(self.window.height());
        self.pending_wrap = false;
    }

    pub(super) fn wind_min(&self) -> i64 {
        pack_crt_coord(self.window.left, self.window.top)
    }

    pub(super) fn wind_max(&self) -> i64 {
        pack_crt_coord(self.window.right, self.window.bottom)
    }

    pub(super) fn abs_x(&self) -> u16 {
        self.window.left + self.cursor_x - 1
    }

    pub(super) fn abs_y(&self) -> u16 {
        self.window.top + self.cursor_y - 1
    }

    pub(super) fn index(&self, x: u16, y: u16) -> usize {
        ((y - 1) * self.width + (x - 1)) as usize
    }

    fn blank_cell(&self) -> ScreenCell {
        ScreenCell {
            glyph: " ".into(),
            fg: self.active_fg,
            bg: self.active_bg,
            continuation: false,
        }
    }

    pub(super) fn set_window(&mut self, window: WindowRect) {
        self.window = window;
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.pending_wrap = false;
    }

    pub(super) fn clip_window(&self, window: WindowRect) -> Option<WindowRect> {
        Some(window)
    }

    pub(super) fn can_paint_cell(&self, _x: u16, _y: u16) -> bool {
        true
    }

    pub(super) fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.pending_wrap = false;
    }

    pub(super) fn cell_at(&self, x: u16, y: u16) -> ScreenCell {
        self.cells[self.index(x, y)].clone()
    }

    /// Writes one CRT cell using packed palette colors (`0..=15`), bypassing the cursor.
    pub(super) fn paint_packed_cell(&mut self, x: u16, y: u16, ch: char, fg: u8, bg: u8) {
        if x == 0 || y == 0 || x > self.width || y > self.height {
            return;
        }
        let idx = self.index(x, y);
        self.cells[idx] = ScreenCell {
            glyph: ch.to_string(),
            fg: RenderColor::Crt(fg.min(15)),
            bg: RenderColor::Crt(bg.min(15)),
            continuation: false,
        };
    }

    pub(super) fn use_packed_colors(&mut self) {
        self.active_fg = RenderColor::Crt(self.fg);
        self.active_bg = RenderColor::Crt(self.bg);
    }

    pub(super) fn set_extended_fg_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.active_fg = RenderColor::Rgb { r, g, b };
    }

    pub(super) fn set_extended_bg_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.active_bg = RenderColor::Rgb { r, g, b };
    }

    pub(super) fn set_extended_fg_ansi(&mut self, index: u8) {
        self.active_fg = RenderColor::Ansi256(index);
    }

    pub(super) fn set_extended_bg_ansi(&mut self, index: u8) {
        self.active_bg = RenderColor::Ansi256(index);
    }

    pub(super) fn width(&self) -> u16 {
        self.width
    }

    pub(super) fn height(&self) -> u16 {
        self.height
    }

    /// Returns the character content of one screen row (full width, space-padded).
    pub(super) fn row_text(&self, y: u16) -> String {
        (1..=self.width)
            .map(|x| self.cells[self.index(x, y)].glyph.as_str())
            .collect()
    }

    /// Returns one CRT cell (`x`/`y` one-based) when both colors are packed palette indices.
    pub(super) fn packed_cell_at(&self, x: u16, y: u16) -> Option<(char, u8, u8)> {
        let cell = self.cell_at(x, y);
        let fg = cell.fg.packed_index()?;
        let bg = cell.bg.packed_index()?;
        let mut chars = cell.glyph.chars();
        let ch = match (chars.next(), chars.next()) {
            (Some(ch), None) => ch,
            _ => return None,
        };
        Some((ch, fg, bg))
    }

    #[cfg(test)]
    pub(super) fn cell_at_packed(&self, x: u16, y: u16) -> (char, u8, u8) {
        self.packed_cell_at(x, y)
            .expect("expected packed CRT colors")
    }

    #[cfg(test)]
    pub(super) fn line_text(&self, y: u16) -> String {
        self.row_text(y)
    }

    #[cfg(test)]
    pub(super) fn cell_color_labels(&self, x: u16, y: u16) -> (char, String, String) {
        let cell = self.cell_at(x, y);
        let ch = cell.glyph.chars().next().unwrap_or(' ');
        (ch, cell.fg.debug_label(), cell.bg.debug_label())
    }
}

fn pack_crt_coord(x: u16, y: u16) -> i64 {
    let x = i64::from(x & 0x00FF);
    let y = i64::from(y & 0x00FF);
    x | (y << 8)
}
