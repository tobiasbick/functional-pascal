use crossterm::style::Color;

pub(super) const DEFAULT_SCREEN_WIDTH: u16 = 80;
pub(super) const DEFAULT_SCREEN_HEIGHT: u16 = 25;
pub(super) const TEXT_MODE_BW40: i64 = 0;
pub(super) const TEXT_MODE_C40: i64 = 1;
pub(super) const TEXT_MODE_BW80: i64 = 2;
pub(super) const TEXT_MODE_C80: i64 = 3;
pub(super) const TEXT_MODE_CO40: i64 = 4;
pub(super) const TEXT_MODE_CO80: i64 = 5;
pub(super) const TEXT_MODE_MONO: i64 = 7;

mod frames;
mod writing;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderColor {
    Crt(u8),
    Rgb { r: u8, g: u8, b: u8 },
    Ansi256(u8),
}

impl RenderColor {
    pub(super) fn to_crossterm(self) -> Color {
        match self {
            Self::Crt(index) => match index {
                0 => Color::Black,
                1 => Color::DarkBlue,
                2 => Color::DarkGreen,
                3 => Color::DarkCyan,
                4 => Color::DarkRed,
                5 => Color::DarkMagenta,
                6 => Color::DarkYellow,
                7 => Color::Grey,
                8 => Color::DarkGrey,
                9 => Color::Blue,
                10 => Color::Green,
                11 => Color::Cyan,
                12 => Color::Red,
                13 => Color::Magenta,
                14 => Color::Yellow,
                _ => Color::White,
            },
            Self::Rgb { r, g, b } => Color::Rgb { r, g, b },
            Self::Ansi256(index) => Color::AnsiValue(index),
        }
    }

    #[cfg(test)]
    fn packed_index(self) -> Option<u8> {
        match self {
            Self::Crt(index) => Some(index),
            Self::Rgb { .. } | Self::Ansi256(_) => None,
        }
    }

    #[cfg(test)]
    fn debug_label(self) -> String {
        match self {
            Self::Crt(index) => format!("crt:{index}"),
            Self::Rgb { r, g, b } => format!("rgb:{r},{g},{b}"),
            Self::Ansi256(index) => format!("ansi256:{index}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScreenCell {
    pub(super) ch: char,
    pub(super) fg: RenderColor,
    pub(super) bg: RenderColor,
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
}

impl ConsoleState {
    pub(super) fn new(width: u16, height: u16) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let blank = ScreenCell {
            ch: ' ',
            fg: RenderColor::Crt(7),
            bg: RenderColor::Crt(0),
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
            ch: ' ',
            fg: self.active_fg,
            bg: self.active_bg,
        };
        let old_width = self.width;
        let old_height = self.height;
        let old_cells = self.cells.clone();
        let mut new_cells = vec![blank; new_width as usize * new_height as usize];

        for y in 1..=old_height.min(new_height) {
            for x in 1..=old_width.min(new_width) {
                let old_idx = ((y - 1) * old_width + (x - 1)) as usize;
                let new_idx = ((y - 1) * new_width + (x - 1)) as usize;
                new_cells[new_idx] = old_cells[old_idx];
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.cells = new_cells;
        self.prev_cells.clear();

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
            ch: ' ',
            fg: self.active_fg,
            bg: self.active_bg,
        }
    }

    pub(super) fn set_window(&mut self, window: WindowRect) {
        self.window = window;
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.pending_wrap = false;
    }

    pub(super) fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.pending_wrap = false;
    }

    pub(super) fn cell_at(&self, x: u16, y: u16) -> ScreenCell {
        self.cells[self.index(x, y)]
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

    #[cfg(test)]
    pub(super) fn line_text(&self, y: u16) -> String {
        (1..=self.width)
            .map(|x| self.cells[self.index(x, y)].ch)
            .collect()
    }

    #[cfg(test)]
    pub(super) fn cell_at_packed(&self, x: u16, y: u16) -> (char, u8, u8) {
        let cell = self.cell_at(x, y);
        let fg = cell.fg.packed_index();
        let bg = cell.bg.packed_index();
        assert!(fg.is_some(), "expected packed foreground color");
        assert!(bg.is_some(), "expected packed background color");
        (cell.ch, fg.unwrap_or_default(), bg.unwrap_or_default())
    }

    #[cfg(test)]
    pub(super) fn cell_color_labels(&self, x: u16, y: u16) -> (char, String, String) {
        let cell = self.cell_at(x, y);
        (cell.ch, cell.fg.debug_label(), cell.bg.debug_label())
    }
}

fn pack_crt_coord(x: u16, y: u16) -> i64 {
    let x = i64::from(x & 0x00FF);
    let y = i64::from(y & 0x00FF);
    x | (y << 8)
}
