use super::super::Console;
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

impl Console {
    pub fn goto_xy(&mut self, x: i64, y: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let (Some(x), Some(y)) = (
            self.check_coord(x, self.state.window_width()),
            self.check_coord(y, self.state.window_height()),
        ) else {
            return Ok(());
        };
        self.state.set_cursor(x, y);
        self.render_if_ready(location)
    }

    pub fn where_x(&self) -> i64 {
        i64::from(self.state.cursor_x)
    }

    pub fn where_y(&self) -> i64 {
        i64::from(self.state.cursor_y)
    }

    pub fn cursor_on(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = true;
        self.state.cursor_big = false;
        self.render_if_ready(location)
    }

    pub fn cursor_off(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = false;
        self.render_if_ready(location)
    }

    pub fn cursor_big(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = true;
        self.state.cursor_big = true;
        self.render_if_ready(location)
    }
}
