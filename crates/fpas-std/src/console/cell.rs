//! Public value types for cell-oriented `Std.Console` drawing.
//!
//! **Documentation:** `docs/pascal/std/console/cells-frames.md`.

/// Ordered variants of `Std.Console.ColorKind`.
pub const CONSOLE_COLOR_KIND_VARIANTS: &[&str] = &["Crt", "Ansi256", "Rgb"];

/// A terminal color stored in a console cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleColor {
    /// One of the classic 16 CRT palette colors.
    Crt(u8),
    /// One of the ANSI 256-color palette entries.
    Ansi256(u8),
    /// A 24-bit RGB color.
    Rgb {
        /// Red channel.
        red: u8,
        /// Green channel.
        green: u8,
        /// Blue channel.
        blue: u8,
    },
}

/// One logical terminal glyph and its foreground/background colors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCell {
    /// The one extended grapheme cluster painted at the cell.
    pub glyph: String,
    /// Foreground color.
    pub foreground: ConsoleColor,
    /// Background color.
    pub background: ConsoleColor,
}

/// A 1-based, screen-absolute rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleRect {
    /// Left column.
    pub x: u16,
    /// Top row.
    pub y: u16,
    /// Width in terminal cells.
    pub width: u16,
    /// Height in terminal cells.
    pub height: u16,
}

/// Opaque identifier for a saved console region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SavedRegionId(pub u64);
