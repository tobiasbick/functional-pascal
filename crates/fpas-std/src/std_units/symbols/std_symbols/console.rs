//! `Std.Console` symbol names and registry group.

pub const STD_CONSOLE_WRITE_LN: &str = std_console!("WriteLn");
pub const STD_CONSOLE_WRITE: &str = std_console!("Write");
pub const STD_CONSOLE_CLR_SCR: &str = std_console!("ClrScr");
pub const STD_CONSOLE_CLR_EOL: &str = std_console!("ClrEol");
pub const STD_CONSOLE_GOTO_XY: &str = std_console!("GotoXY");
pub const STD_CONSOLE_WHERE_X: &str = std_console!("WhereX");
pub const STD_CONSOLE_WHERE_Y: &str = std_console!("WhereY");
pub const STD_CONSOLE_WIND_MIN: &str = std_console!("WindMin");
pub const STD_CONSOLE_WIND_MAX: &str = std_console!("WindMax");
pub const STD_CONSOLE_DEL_LINE: &str = std_console!("DelLine");
pub const STD_CONSOLE_INS_LINE: &str = std_console!("InsLine");
pub const STD_CONSOLE_WINDOW: &str = std_console!("Window");
pub const STD_CONSOLE_TEXT_COLOR: &str = std_console!("TextColor");
pub const STD_CONSOLE_TEXT_BACKGROUND: &str = std_console!("TextBackground");
pub const STD_CONSOLE_HIGH_VIDEO: &str = std_console!("HighVideo");
pub const STD_CONSOLE_LOW_VIDEO: &str = std_console!("LowVideo");
pub const STD_CONSOLE_NORM_VIDEO: &str = std_console!("NormVideo");
pub const STD_CONSOLE_TEXT_ATTR: &str = std_console!("TextAttr");
pub const STD_CONSOLE_SET_TEXT_ATTR: &str = std_console!("SetTextAttr");
pub const STD_CONSOLE_DELAY: &str = std_console!("Delay");
pub const STD_CONSOLE_CURSOR_ON: &str = std_console!("CursorOn");
pub const STD_CONSOLE_CURSOR_OFF: &str = std_console!("CursorOff");
pub const STD_CONSOLE_CURSOR_BIG: &str = std_console!("CursorBig");
pub const STD_CONSOLE_TEXT_MODE: &str = std_console!("TextMode");
pub const STD_CONSOLE_LAST_MODE: &str = std_console!("LastMode");
pub const STD_CONSOLE_SCREEN_WIDTH: &str = std_console!("ScreenWidth");
pub const STD_CONSOLE_SCREEN_HEIGHT: &str = std_console!("ScreenHeight");
pub const STD_CONSOLE_SOUND: &str = std_console!("Sound");
pub const STD_CONSOLE_NO_SOUND: &str = std_console!("NoSound");
pub const STD_CONSOLE_ASSIGN_CRT: &str = std_console!("AssignCrt");
pub const STD_CONSOLE_READ_LN: &str = std_console!("ReadLn");
pub const STD_CONSOLE_READ: &str = std_console!("Read");
pub const STD_CONSOLE_READ_KEY: &str = std_console!("ReadKey");
pub const STD_CONSOLE_KEY_PRESSED: &str = std_console!("KeyPressed");
pub const STD_CONSOLE_READ_KEY_EVENT: &str = std_console!("ReadKeyEvent");
pub const STD_CONSOLE_EVENT_PENDING: &str = std_console!("EventPending");
pub const STD_CONSOLE_READ_EVENT: &str = std_console!("ReadEvent");
pub const STD_CONSOLE_READ_EVENT_TIMEOUT: &str = std_console!("ReadEventTimeout");
pub const STD_CONSOLE_POLL_EVENT: &str = std_console!("PollEvent");
pub const STD_CONSOLE_KEY_EVENT: &str = std_console!("KeyEvent");
pub const STD_CONSOLE_KEY_KIND: &str = std_console!("KeyKind");
pub const STD_CONSOLE_EVENT: &str = std_console!("Event");
pub const STD_CONSOLE_EVENT_KIND: &str = std_console!("EventKind");
pub const STD_CONSOLE_MOUSE_ACTION: &str = std_console!("MouseAction");
pub const STD_CONSOLE_MOUSE_BUTTON: &str = std_console!("MouseButton");
pub const STD_CONSOLE_ENABLE_RAW_MODE: &str = std_console!("EnableRawMode");
pub const STD_CONSOLE_DISABLE_RAW_MODE: &str = std_console!("DisableRawMode");
pub const STD_CONSOLE_ENTER_ALT_SCREEN: &str = std_console!("EnterAltScreen");
pub const STD_CONSOLE_LEAVE_ALT_SCREEN: &str = std_console!("LeaveAltScreen");
pub const STD_CONSOLE_ENABLE_MOUSE: &str = std_console!("EnableMouse");
pub const STD_CONSOLE_DISABLE_MOUSE: &str = std_console!("DisableMouse");
pub const STD_CONSOLE_ENABLE_FOCUS: &str = std_console!("EnableFocus");
pub const STD_CONSOLE_DISABLE_FOCUS: &str = std_console!("DisableFocus");
pub const STD_CONSOLE_ENABLE_PASTE: &str = std_console!("EnablePaste");
pub const STD_CONSOLE_DISABLE_PASTE: &str = std_console!("DisablePaste");
pub const STD_CONSOLE_BLACK: &str = std_console!("Black");
pub const STD_CONSOLE_BLUE: &str = std_console!("Blue");
pub const STD_CONSOLE_GREEN: &str = std_console!("Green");
pub const STD_CONSOLE_CYAN: &str = std_console!("Cyan");
pub const STD_CONSOLE_RED: &str = std_console!("Red");
pub const STD_CONSOLE_MAGENTA: &str = std_console!("Magenta");
pub const STD_CONSOLE_BROWN: &str = std_console!("Brown");
pub const STD_CONSOLE_LIGHT_GRAY: &str = std_console!("LightGray");
pub const STD_CONSOLE_DARK_GRAY: &str = std_console!("DarkGray");
pub const STD_CONSOLE_LIGHT_BLUE: &str = std_console!("LightBlue");
pub const STD_CONSOLE_LIGHT_GREEN: &str = std_console!("LightGreen");
pub const STD_CONSOLE_LIGHT_CYAN: &str = std_console!("LightCyan");
pub const STD_CONSOLE_LIGHT_RED: &str = std_console!("LightRed");
pub const STD_CONSOLE_LIGHT_MAGENTA: &str = std_console!("LightMagenta");
pub const STD_CONSOLE_YELLOW: &str = std_console!("Yellow");
pub const STD_CONSOLE_WHITE: &str = std_console!("White");
pub const STD_CONSOLE_TEXT_COLOR_RGB: &str = std_console!("TextColorRGB");
pub const STD_CONSOLE_TEXT_BACKGROUND_RGB: &str = std_console!("TextBackgroundRGB");
pub const STD_CONSOLE_TEXT_COLOR_256: &str = std_console!("TextColor256");
pub const STD_CONSOLE_TEXT_BACKGROUND_256: &str = std_console!("TextBackground256");
pub const STD_CONSOLE_BLINK: &str = std_console!("Blink");
pub const STD_CONSOLE_BW40: &str = std_console!("BW40");
pub const STD_CONSOLE_C40: &str = std_console!("C40");
pub const STD_CONSOLE_BW80: &str = std_console!("BW80");
pub const STD_CONSOLE_C80: &str = std_console!("C80");
pub const STD_CONSOLE_CO40: &str = std_console!("CO40");
pub const STD_CONSOLE_CO80: &str = std_console!("CO80");
pub const STD_CONSOLE_MONO: &str = std_console!("Mono");
pub const STD_CONSOLE_FONT_8X8: &str = std_console!("Font8x8");
/// Qualified name of the `Std.Console.ColorKind` enum.
pub const STD_CONSOLE_COLOR_KIND: &str = std_console!("ColorKind");
/// Qualified name of the `Std.Console.Color` record.
pub const STD_CONSOLE_COLOR: &str = std_console!("Color");
/// Qualified name of the `Std.Console.Cell` record.
pub const STD_CONSOLE_CELL: &str = std_console!("Cell");
/// Qualified name of the `Std.Console.Rect` record.
pub const STD_CONSOLE_RECT: &str = std_console!("Rect");
/// Qualified name of the opaque `Std.Console.SavedRegion` record.
pub const STD_CONSOLE_SAVED_REGION: &str = std_console!("SavedRegion");
/// Qualified name of the `Std.Console.CrtColor` constructor.
pub const STD_CONSOLE_CRT_COLOR: &str = std_console!("CrtColor");
/// Qualified name of the `Std.Console.Ansi256Color` constructor.
pub const STD_CONSOLE_ANSI_256_COLOR: &str = std_console!("Ansi256Color");
/// Qualified name of the `Std.Console.RgbColor` constructor.
pub const STD_CONSOLE_RGB_COLOR: &str = std_console!("RgbColor");
/// Qualified name of `Std.Console.BeginFrame`.
pub const STD_CONSOLE_BEGIN_FRAME: &str = std_console!("BeginFrame");
/// Qualified name of `Std.Console.Present`.
pub const STD_CONSOLE_PRESENT: &str = std_console!("Present");
/// Qualified name of `Std.Console.PutCell`.
pub const STD_CONSOLE_PUT_CELL: &str = std_console!("PutCell");
/// Qualified name of `Std.Console.GetCell`.
pub const STD_CONSOLE_GET_CELL: &str = std_console!("GetCell");
/// Qualified name of `Std.Console.FillRect`.
pub const STD_CONSOLE_FILL_RECT: &str = std_console!("FillRect");
/// Qualified name of `Std.Console.WriteCells`.
pub const STD_CONSOLE_WRITE_CELLS: &str = std_console!("WriteCells");
/// Qualified name of `Std.Console.SaveRegion`.
pub const STD_CONSOLE_SAVE_REGION: &str = std_console!("SaveRegion");
/// Qualified name of `Std.Console.RestoreRegion`.
pub const STD_CONSOLE_RESTORE_REGION: &str = std_console!("RestoreRegion");
/// Qualified name of `Std.Console.DiscardRegion`.
pub const STD_CONSOLE_DISCARD_REGION: &str = std_console!("DiscardRegion");
/// Qualified name of `Std.Console.DisplayWidth`.
pub const STD_CONSOLE_DISPLAY_WIDTH: &str = std_console!("DisplayWidth");
/// Qualified name of `Std.Console.GraphemeWidth`.
pub const STD_CONSOLE_GRAPHEME_WIDTH: &str = std_console!("GraphemeWidth");
/// Qualified name of `Std.Console.SplitGraphemes`.
pub const STD_CONSOLE_SPLIT_GRAPHEMES: &str = std_console!("SplitGraphemes");

/// Symbols exported by the `Std.Console` unit.
pub(in crate::std_units) const STD_CONSOLE_SYMBOLS: &[&str] = &[
    STD_CONSOLE_WRITE_LN,
    STD_CONSOLE_WRITE,
    STD_CONSOLE_CLR_SCR,
    STD_CONSOLE_CLR_EOL,
    STD_CONSOLE_GOTO_XY,
    STD_CONSOLE_WHERE_X,
    STD_CONSOLE_WHERE_Y,
    STD_CONSOLE_WIND_MIN,
    STD_CONSOLE_WIND_MAX,
    STD_CONSOLE_DEL_LINE,
    STD_CONSOLE_INS_LINE,
    STD_CONSOLE_WINDOW,
    STD_CONSOLE_TEXT_COLOR,
    STD_CONSOLE_TEXT_BACKGROUND,
    STD_CONSOLE_TEXT_COLOR_RGB,
    STD_CONSOLE_TEXT_BACKGROUND_RGB,
    STD_CONSOLE_TEXT_COLOR_256,
    STD_CONSOLE_TEXT_BACKGROUND_256,
    STD_CONSOLE_HIGH_VIDEO,
    STD_CONSOLE_LOW_VIDEO,
    STD_CONSOLE_NORM_VIDEO,
    STD_CONSOLE_TEXT_ATTR,
    STD_CONSOLE_SET_TEXT_ATTR,
    STD_CONSOLE_DELAY,
    STD_CONSOLE_CURSOR_ON,
    STD_CONSOLE_CURSOR_OFF,
    STD_CONSOLE_CURSOR_BIG,
    STD_CONSOLE_TEXT_MODE,
    STD_CONSOLE_LAST_MODE,
    STD_CONSOLE_SCREEN_WIDTH,
    STD_CONSOLE_SCREEN_HEIGHT,
    STD_CONSOLE_SOUND,
    STD_CONSOLE_NO_SOUND,
    STD_CONSOLE_ASSIGN_CRT,
    STD_CONSOLE_READ_LN,
    STD_CONSOLE_READ,
    STD_CONSOLE_READ_KEY,
    STD_CONSOLE_KEY_PRESSED,
    STD_CONSOLE_READ_KEY_EVENT,
    STD_CONSOLE_EVENT_PENDING,
    STD_CONSOLE_READ_EVENT,
    STD_CONSOLE_KEY_EVENT,
    STD_CONSOLE_KEY_KIND,
    STD_CONSOLE_EVENT,
    STD_CONSOLE_EVENT_KIND,
    STD_CONSOLE_MOUSE_ACTION,
    STD_CONSOLE_MOUSE_BUTTON,
    STD_CONSOLE_ENABLE_RAW_MODE,
    STD_CONSOLE_DISABLE_RAW_MODE,
    STD_CONSOLE_ENTER_ALT_SCREEN,
    STD_CONSOLE_LEAVE_ALT_SCREEN,
    STD_CONSOLE_ENABLE_MOUSE,
    STD_CONSOLE_DISABLE_MOUSE,
    STD_CONSOLE_ENABLE_FOCUS,
    STD_CONSOLE_DISABLE_FOCUS,
    STD_CONSOLE_ENABLE_PASTE,
    STD_CONSOLE_DISABLE_PASTE,
    STD_CONSOLE_READ_EVENT_TIMEOUT,
    STD_CONSOLE_POLL_EVENT,
    STD_CONSOLE_BLACK,
    STD_CONSOLE_BLUE,
    STD_CONSOLE_GREEN,
    STD_CONSOLE_CYAN,
    STD_CONSOLE_RED,
    STD_CONSOLE_MAGENTA,
    STD_CONSOLE_BROWN,
    STD_CONSOLE_LIGHT_GRAY,
    STD_CONSOLE_DARK_GRAY,
    STD_CONSOLE_LIGHT_BLUE,
    STD_CONSOLE_LIGHT_GREEN,
    STD_CONSOLE_LIGHT_CYAN,
    STD_CONSOLE_LIGHT_RED,
    STD_CONSOLE_LIGHT_MAGENTA,
    STD_CONSOLE_YELLOW,
    STD_CONSOLE_WHITE,
    STD_CONSOLE_BLINK,
    STD_CONSOLE_BW40,
    STD_CONSOLE_C40,
    STD_CONSOLE_BW80,
    STD_CONSOLE_C80,
    STD_CONSOLE_CO40,
    STD_CONSOLE_CO80,
    STD_CONSOLE_MONO,
    STD_CONSOLE_FONT_8X8,
    STD_CONSOLE_COLOR_KIND,
    STD_CONSOLE_COLOR,
    STD_CONSOLE_CELL,
    STD_CONSOLE_RECT,
    STD_CONSOLE_SAVED_REGION,
    STD_CONSOLE_CRT_COLOR,
    STD_CONSOLE_ANSI_256_COLOR,
    STD_CONSOLE_RGB_COLOR,
    STD_CONSOLE_BEGIN_FRAME,
    STD_CONSOLE_PRESENT,
    STD_CONSOLE_PUT_CELL,
    STD_CONSOLE_GET_CELL,
    STD_CONSOLE_FILL_RECT,
    STD_CONSOLE_WRITE_CELLS,
    STD_CONSOLE_SAVE_REGION,
    STD_CONSOLE_RESTORE_REGION,
    STD_CONSOLE_DISCARD_REGION,
    STD_CONSOLE_DISPLAY_WIDTH,
    STD_CONSOLE_GRAPHEME_WIDTH,
    STD_CONSOLE_SPLIT_GRAPHEMES,
];
