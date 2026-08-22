use super::super::Console;
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

impl Console {
    /// `Std.Console.TextColor(Color)` — select a packed CRT foreground color.
    pub fn text_color(&mut self, color: i64, location: SourceLocation) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.state.fg = self.validate_color(color, "TextColor", location)?;
        self.state.use_packed_colors();
        Ok(())
    }

    /// `Std.Console.TextBackground(Color)` — select a packed CRT background color.
    pub fn text_background(
        &mut self,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.state.bg = self.validate_color(color, "TextBackground", location)?;
        self.state.use_packed_colors();
        Ok(())
    }

    /// `Std.Console.HighVideo()` — enable the packed bright foreground bit.
    pub fn high_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg |= 0x08;
        self.state.use_packed_colors();
        self.render_if_ready(location)
    }

    /// `Std.Console.LowVideo()` — disable the packed bright foreground bit.
    pub fn low_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg &= 0x07;
        self.state.use_packed_colors();
        self.render_if_ready(location)
    }

    /// `Std.Console.NormVideo()` — restore packed light-gray-on-black colors.
    pub fn norm_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg = 7;
        self.state.bg = 0;
        self.state.use_packed_colors();
        self.render_if_ready(location)
    }

    /// Returns the packed CRT foreground/background attribute.
    pub fn text_attr(&self) -> i64 {
        i64::from((self.state.bg << 4) | (self.state.fg & 0x0F))
    }

    /// `Std.Console.SetTextAttr(Attr)` — restore packed CRT colors from `Attr`.
    pub fn set_text_attr(&mut self, attr: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let attr = self.validate_text_attr(attr, location)?;
        self.state.fg = attr & 0x0F;
        self.state.bg = (attr >> 4) & 0x0F;
        self.state.use_packed_colors();
        self.render_if_ready(location)
    }

    /// `Std.Console.TextColorRGB(R, G, B)` — set fg to 24-bit truecolor.
    ///
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn text_color_rgb(
        &mut self,
        r: i64,
        g: i64,
        b: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let (r, g, b) = self.validate_rgb(r, g, b, "TextColorRGB", location)?;
        self.state.set_extended_fg_rgb(r, g, b);
        if self.state.crt_mode {
            return Ok(());
        }
        self.run_writer_command(
            crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r, g, b }),
            "TextColorRGB failed",
            location,
        )
    }

    /// `Std.Console.TextBackgroundRGB(R, G, B)` — set bg to 24-bit truecolor.
    ///
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn text_background_rgb(
        &mut self,
        r: i64,
        g: i64,
        b: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let (r, g, b) = self.validate_rgb(r, g, b, "TextBackgroundRGB", location)?;
        self.state.set_extended_bg_rgb(r, g, b);
        if self.state.crt_mode {
            return Ok(());
        }
        self.run_writer_command(
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r, g, b }),
            "TextBackgroundRGB failed",
            location,
        )
    }

    /// `Std.Console.TextColor256(Index)` — set fg to 256-color palette index (0–255).
    ///
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn text_color_256(&mut self, index: i64, location: SourceLocation) -> Result<(), StdError> {
        let index = self.validate_color_256(index, "TextColor256", location)?;
        self.state.set_extended_fg_ansi(index);
        if self.state.crt_mode {
            return Ok(());
        }
        self.run_writer_command(
            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(index)),
            "TextColor256 failed",
            location,
        )
    }

    /// `Std.Console.TextBackground256(Index)` — set bg to 256-color palette index (0–255).
    ///
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn text_background_256(
        &mut self,
        index: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let index = self.validate_color_256(index, "TextBackground256", location)?;
        self.state.set_extended_bg_ansi(index);
        if self.state.crt_mode {
            return Ok(());
        }
        self.run_writer_command(
            crossterm::style::SetBackgroundColor(crossterm::style::Color::AnsiValue(index)),
            "TextBackground256 failed",
            location,
        )
    }
}
