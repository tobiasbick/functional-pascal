use super::{ConsoleState, FrameDamage, RenderColor, ScreenCell, WindowRect};
use crate::console::cell::{ConsoleCell, ConsoleColor, ConsoleRect};
use crate::text::cell_width::grapheme_cell_width;

impl From<ConsoleColor> for RenderColor {
    fn from(value: ConsoleColor) -> Self {
        match value {
            ConsoleColor::Crt(index) => Self::Crt(index),
            ConsoleColor::Ansi256(index) => Self::Ansi256(index),
            ConsoleColor::Rgb { red, green, blue } => Self::Rgb {
                r: red,
                g: green,
                b: blue,
            },
        }
    }
}

impl From<RenderColor> for ConsoleColor {
    fn from(value: RenderColor) -> Self {
        match value {
            RenderColor::Crt(index) => Self::Crt(index),
            RenderColor::Ansi256(index) => Self::Ansi256(index),
            RenderColor::Rgb { r, g, b } => Self::Rgb {
                red: r,
                green: g,
                blue: b,
            },
        }
    }
}

impl ConsoleState {
    pub(in super::super) fn public_cell_at(&self, x: u16, y: u16) -> Option<ConsoleCell> {
        if !self.contains(x, y) {
            return None;
        }
        let cell = self.cell_at(x, y);
        if cell.continuation {
            return None;
        }
        Some(ConsoleCell {
            glyph: cell.glyph.clone(),
            foreground: cell.fg.into(),
            background: cell.bg.into(),
        })
    }

    pub(in super::super) fn put_cell(&mut self, x: u16, y: u16, cell: ConsoleCell) -> bool {
        let Some(dirty) = self.put_cell_untracked(x, y, cell) else {
            return false;
        };
        self.mark_damage_rect(dirty);
        true
    }

    fn put_cell_untracked(&mut self, x: u16, y: u16, cell: ConsoleCell) -> Option<WindowRect> {
        let width = u16::from(grapheme_cell_width(&cell.glyph).unwrap_or(0));
        if width == 0 || !self.contains(x, y) || x.saturating_add(width - 1) > self.width {
            return None;
        }

        let mut left = x;
        let write_right = x + width - 1;
        let mut right = write_right;
        for column in x..=write_right {
            let repaired = self.repair_wide_cell(column, y);
            left = left.min(repaired.left);
            right = right.max(repaired.right);
        }

        let fg = cell.foreground.into();
        let bg = cell.background.into();
        let index = self.index(x, y);
        self.cells[index] = ScreenCell {
            glyph: cell.glyph,
            fg,
            bg,
            continuation: false,
        };
        if width == 2 {
            let continuation = self.index(x + 1, y);
            self.cells[continuation] = ScreenCell {
                glyph: " ".into(),
                fg,
                bg,
                continuation: true,
            };
        }
        Some(WindowRect {
            left,
            top: y,
            right,
            bottom: y,
        })
    }

    pub(in super::super) fn fill_rect(&mut self, rect: ConsoleRect, cell: ConsoleCell) {
        let Some(rect) = self.public_rect(rect) else {
            return;
        };
        let mut damage: Option<WindowRect> = None;
        for y in rect.top..=rect.bottom {
            for x in rect.left..=rect.right {
                if let Some(dirty) = self.put_cell_untracked(x, y, cell.clone()) {
                    damage = Some(match damage {
                        Some(existing) => existing.union(dirty),
                        None => dirty,
                    });
                }
            }
        }
        if let Some(damage) = damage {
            self.mark_damage_rect(damage);
        }
    }

    pub(in super::super) fn write_cells(&mut self, start_x: u16, y: u16, cells: &[ConsoleCell]) {
        let mut x = start_x;
        let mut damage: Option<WindowRect> = None;
        for cell in cells {
            let width = u16::from(grapheme_cell_width(&cell.glyph).unwrap_or(0));
            if width == 0 {
                continue;
            }
            if x > self.width || x.saturating_add(width - 1) > self.width {
                break;
            }
            if let Some(dirty) = self.put_cell_untracked(x, y, cell.clone()) {
                damage = Some(match damage {
                    Some(existing) => existing.union(dirty),
                    None => dirty,
                });
            }
            x = x.saturating_add(width);
        }
        if let Some(damage) = damage {
            self.mark_damage_rect(damage);
        }
    }

    pub(in super::super) fn public_rect(&self, rect: ConsoleRect) -> Option<WindowRect> {
        if rect.x == 0 || rect.y == 0 || rect.width == 0 || rect.height == 0 {
            return None;
        }
        let right = rect.x.saturating_add(rect.width - 1).min(self.width);
        let bottom = rect.y.saturating_add(rect.height - 1).min(self.height);
        if rect.x > self.width || rect.y > self.height {
            return None;
        }
        Some(WindowRect {
            left: rect.x,
            top: rect.y,
            right,
            bottom,
        })
    }

    pub(in super::super) fn normalize_wide_cells(&mut self) {
        let mut changed = false;
        for y in 1..=self.height {
            for x in 1..=self.width {
                let index = self.index(x, y);
                let cell = self.cells[index].clone();
                if cell.continuation {
                    let valid = x > 1
                        && grapheme_cell_width(&self.cells[self.index(x - 1, y)].glyph) == Some(2)
                        && !self.cells[self.index(x - 1, y)].continuation;
                    if !valid {
                        self.cells[index] = ScreenCell {
                            glyph: " ".into(),
                            fg: cell.fg,
                            bg: cell.bg,
                            continuation: false,
                        };
                        changed = true;
                    }
                } else if grapheme_cell_width(&cell.glyph) == Some(2) {
                    if x == self.width {
                        self.cells[index] = ScreenCell {
                            glyph: " ".into(),
                            fg: cell.fg,
                            bg: cell.bg,
                            continuation: false,
                        };
                        changed = true;
                    } else {
                        let next = self.index(x + 1, y);
                        let continuation = ScreenCell {
                            glyph: " ".into(),
                            fg: cell.fg,
                            bg: cell.bg,
                            continuation: true,
                        };
                        if self.cells[next] != continuation {
                            self.cells[next] = continuation;
                            changed = true;
                        }
                    }
                }
            }
        }
        if changed {
            self.pending_frame_damage = Some(FrameDamage::FullFrame);
        }
    }

    fn contains(&self, x: u16, y: u16) -> bool {
        x > 0 && y > 0 && x <= self.width && y <= self.height
    }

    fn repair_wide_cell(&mut self, x: u16, y: u16) -> WindowRect {
        let current = self.cell_at(x, y);
        let mut left = x;
        let mut right = x;

        if current.continuation && x > 1 {
            left = x - 1;
            let index = self.index(left, y);
            let old = self.cells[index].clone();
            self.cells[index] = ScreenCell {
                glyph: " ".into(),
                fg: old.fg,
                bg: old.bg,
                continuation: false,
            };
        } else if grapheme_cell_width(&current.glyph) == Some(2) && x < self.width {
            right = x + 1;
            let index = self.index(right, y);
            let old = self.cells[index].clone();
            self.cells[index] = ScreenCell {
                glyph: " ".into(),
                fg: old.fg,
                bg: old.bg,
                continuation: false,
            };
        }

        WindowRect {
            left,
            top: y,
            right,
            bottom: y,
        }
    }
}
