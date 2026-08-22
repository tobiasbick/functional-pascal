//! `Std.Console` symbol names and registry group.

std_symbol!(STD_CONSOLE_WRITE_LN = std_console!("WriteLn"));
std_symbol!(STD_CONSOLE_WRITE = std_console!("Write"));
std_symbol!(STD_CONSOLE_CLR_SCR = std_console!("ClrScr"));
std_symbol!(STD_CONSOLE_CLR_EOL = std_console!("ClrEol"));
std_symbol!(STD_CONSOLE_GOTO_XY = std_console!("GotoXY"));
std_symbol!(STD_CONSOLE_WHERE_X = std_console!("WhereX"));
std_symbol!(STD_CONSOLE_WHERE_Y = std_console!("WhereY"));
std_symbol!(STD_CONSOLE_WIND_MIN = std_console!("WindMin"));
std_symbol!(STD_CONSOLE_WIND_MAX = std_console!("WindMax"));
std_symbol!(STD_CONSOLE_DEL_LINE = std_console!("DelLine"));
std_symbol!(STD_CONSOLE_INS_LINE = std_console!("InsLine"));
std_symbol!(STD_CONSOLE_WINDOW = std_console!("Window"));
std_symbol!(STD_CONSOLE_TEXT_COLOR = std_console!("TextColor"));
std_symbol!(STD_CONSOLE_TEXT_BACKGROUND = std_console!("TextBackground"));
std_symbol!(STD_CONSOLE_HIGH_VIDEO = std_console!("HighVideo"));
std_symbol!(STD_CONSOLE_LOW_VIDEO = std_console!("LowVideo"));
std_symbol!(STD_CONSOLE_NORM_VIDEO = std_console!("NormVideo"));
std_symbol!(STD_CONSOLE_TEXT_ATTR = std_console!("TextAttr"));
std_symbol!(STD_CONSOLE_SET_TEXT_ATTR = std_console!("SetTextAttr"));
std_symbol!(STD_CONSOLE_DELAY = std_console!("Delay"));
std_symbol!(STD_CONSOLE_CURSOR_ON = std_console!("CursorOn"));
std_symbol!(STD_CONSOLE_CURSOR_OFF = std_console!("CursorOff"));
std_symbol!(STD_CONSOLE_CURSOR_BIG = std_console!("CursorBig"));
std_symbol!(STD_CONSOLE_TEXT_MODE = std_console!("TextMode"));
std_symbol!(STD_CONSOLE_LAST_MODE = std_console!("LastMode"));
std_symbol!(STD_CONSOLE_SCREEN_WIDTH = std_console!("ScreenWidth"));
std_symbol!(STD_CONSOLE_SCREEN_HEIGHT = std_console!("ScreenHeight"));
std_symbol!(STD_CONSOLE_SOUND = std_console!("Sound"));
std_symbol!(STD_CONSOLE_NO_SOUND = std_console!("NoSound"));
std_symbol!(STD_CONSOLE_ASSIGN_CRT = std_console!("AssignCrt"));
std_symbol!(STD_CONSOLE_READ_LN = std_console!("ReadLn"));
std_symbol!(STD_CONSOLE_READ = std_console!("Read"));
std_symbol!(STD_CONSOLE_READ_KEY = std_console!("ReadKey"));
std_symbol!(STD_CONSOLE_KEY_PRESSED = std_console!("KeyPressed"));
std_symbol!(STD_CONSOLE_READ_KEY_EVENT = std_console!("ReadKeyEvent"));
std_symbol!(STD_CONSOLE_EVENT_PENDING = std_console!("EventPending"));
std_symbol!(STD_CONSOLE_READ_EVENT = std_console!("ReadEvent"));
std_symbol!(STD_CONSOLE_READ_EVENT_TIMEOUT = std_console!("ReadEventTimeout"));
std_symbol!(STD_CONSOLE_POLL_EVENT = std_console!("PollEvent"));
std_symbol!(STD_CONSOLE_KEY_EVENT = std_console!("KeyEvent"));
std_symbol!(STD_CONSOLE_KEY_KIND = std_console!("KeyKind"));
std_symbol!(STD_CONSOLE_EVENT = std_console!("Event"));
std_symbol!(STD_CONSOLE_EVENT_KIND = std_console!("EventKind"));
std_symbol!(STD_CONSOLE_MOUSE_ACTION = std_console!("MouseAction"));
std_symbol!(STD_CONSOLE_MOUSE_BUTTON = std_console!("MouseButton"));
std_symbol!(STD_CONSOLE_ENABLE_RAW_MODE = std_console!("EnableRawMode"));
std_symbol!(STD_CONSOLE_DISABLE_RAW_MODE = std_console!("DisableRawMode"));
std_symbol!(STD_CONSOLE_ENTER_ALT_SCREEN = std_console!("EnterAltScreen"));
std_symbol!(STD_CONSOLE_LEAVE_ALT_SCREEN = std_console!("LeaveAltScreen"));
std_symbol!(STD_CONSOLE_ENABLE_MOUSE = std_console!("EnableMouse"));
std_symbol!(STD_CONSOLE_DISABLE_MOUSE = std_console!("DisableMouse"));
std_symbol!(STD_CONSOLE_ENABLE_FOCUS = std_console!("EnableFocus"));
std_symbol!(STD_CONSOLE_DISABLE_FOCUS = std_console!("DisableFocus"));
std_symbol!(STD_CONSOLE_ENABLE_PASTE = std_console!("EnablePaste"));
std_symbol!(STD_CONSOLE_DISABLE_PASTE = std_console!("DisablePaste"));
std_symbol!(STD_CONSOLE_ACQUIRE_INTERACTIVE_TERMINAL = std_console!("AcquireInteractiveTerminal"));
std_symbol!(STD_CONSOLE_RELEASE_INTERACTIVE_TERMINAL = std_console!("ReleaseInteractiveTerminal"));
std_symbol!(STD_CONSOLE_BLACK = std_console!("Black"));
std_symbol!(STD_CONSOLE_BLUE = std_console!("Blue"));
std_symbol!(STD_CONSOLE_GREEN = std_console!("Green"));
std_symbol!(STD_CONSOLE_CYAN = std_console!("Cyan"));
std_symbol!(STD_CONSOLE_RED = std_console!("Red"));
std_symbol!(STD_CONSOLE_MAGENTA = std_console!("Magenta"));
std_symbol!(STD_CONSOLE_BROWN = std_console!("Brown"));
std_symbol!(STD_CONSOLE_LIGHT_GRAY = std_console!("LightGray"));
std_symbol!(STD_CONSOLE_DARK_GRAY = std_console!("DarkGray"));
std_symbol!(STD_CONSOLE_LIGHT_BLUE = std_console!("LightBlue"));
std_symbol!(STD_CONSOLE_LIGHT_GREEN = std_console!("LightGreen"));
std_symbol!(STD_CONSOLE_LIGHT_CYAN = std_console!("LightCyan"));
std_symbol!(STD_CONSOLE_LIGHT_RED = std_console!("LightRed"));
std_symbol!(STD_CONSOLE_LIGHT_MAGENTA = std_console!("LightMagenta"));
std_symbol!(STD_CONSOLE_YELLOW = std_console!("Yellow"));
std_symbol!(STD_CONSOLE_WHITE = std_console!("White"));
std_symbol!(STD_CONSOLE_TEXT_COLOR_RGB = std_console!("TextColorRGB"));
std_symbol!(STD_CONSOLE_TEXT_BACKGROUND_RGB = std_console!("TextBackgroundRGB"));
std_symbol!(STD_CONSOLE_TEXT_COLOR_256 = std_console!("TextColor256"));
std_symbol!(STD_CONSOLE_TEXT_BACKGROUND_256 = std_console!("TextBackground256"));
std_symbol!(STD_CONSOLE_BLINK = std_console!("Blink"));
std_symbol!(STD_CONSOLE_BW40 = std_console!("BW40"));
std_symbol!(STD_CONSOLE_C40 = std_console!("C40"));
std_symbol!(STD_CONSOLE_BW80 = std_console!("BW80"));
std_symbol!(STD_CONSOLE_C80 = std_console!("C80"));
std_symbol!(STD_CONSOLE_CO40 = std_console!("CO40"));
std_symbol!(STD_CONSOLE_CO80 = std_console!("CO80"));
std_symbol!(STD_CONSOLE_MONO = std_console!("Mono"));
std_symbol!(STD_CONSOLE_FONT_8X8 = std_console!("Font8x8"));
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
    STD_CONSOLE_ACQUIRE_INTERACTIVE_TERMINAL,
    STD_CONSOLE_RELEASE_INTERACTIVE_TERMINAL,
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
