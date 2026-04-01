macro_rules! std_console {
    ($suffix:literal) => {
        concat!("Std.Console.", $suffix)
    };
}
macro_rules! std_str {
    ($suffix:literal) => {
        concat!("Std.Str.", $suffix)
    };
}
macro_rules! std_conv {
    ($suffix:literal) => {
        concat!("Std.Conv.", $suffix)
    };
}
macro_rules! std_math {
    ($suffix:literal) => {
        concat!("Std.Math.", $suffix)
    };
}
macro_rules! std_array {
    ($suffix:literal) => {
        concat!("Std.Array.", $suffix)
    };
}
macro_rules! std_result {
    ($suffix:literal) => {
        concat!("Std.Result.", $suffix)
    };
}
macro_rules! std_option {
    ($suffix:literal) => {
        concat!("Std.Option.", $suffix)
    };
}
macro_rules! std_task {
    ($suffix:literal) => {
        concat!("Std.Task.", $suffix)
    };
}
macro_rules! std_dict {
    ($suffix:literal) => {
        concat!("Std.Dict.", $suffix)
    };
}

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
pub const STD_CONSOLE_BLINK: &str = std_console!("Blink");
pub const STD_CONSOLE_BW40: &str = std_console!("BW40");
pub const STD_CONSOLE_C40: &str = std_console!("C40");
pub const STD_CONSOLE_BW80: &str = std_console!("BW80");
pub const STD_CONSOLE_C80: &str = std_console!("C80");
pub const STD_CONSOLE_CO40: &str = std_console!("CO40");
pub const STD_CONSOLE_CO80: &str = std_console!("CO80");
pub const STD_CONSOLE_MONO: &str = std_console!("Mono");
pub const STD_CONSOLE_FONT_8X8: &str = std_console!("Font8x8");

pub const STD_STR_LENGTH: &str = std_str!("Length");
pub const STD_STR_TO_UPPER: &str = std_str!("ToUpper");
pub const STD_STR_TO_LOWER: &str = std_str!("ToLower");
pub const STD_STR_TRIM: &str = std_str!("Trim");
pub const STD_STR_CONTAINS: &str = std_str!("Contains");
pub const STD_STR_STARTS_WITH: &str = std_str!("StartsWith");
pub const STD_STR_ENDS_WITH: &str = std_str!("EndsWith");
pub const STD_STR_SUBSTRING: &str = std_str!("Substring");
pub const STD_STR_INDEX_OF: &str = std_str!("IndexOf");
pub const STD_STR_REPLACE: &str = std_str!("Replace");
pub const STD_STR_SPLIT: &str = std_str!("Split");
pub const STD_STR_JOIN: &str = std_str!("Join");
pub const STD_STR_IS_NUMERIC: &str = std_str!("IsNumeric");
pub const STD_STR_REPEAT: &str = std_str!("RepeatStr");
pub const STD_STR_PAD_LEFT: &str = std_str!("PadLeft");
pub const STD_STR_PAD_RIGHT: &str = std_str!("PadRight");
pub const STD_STR_PAD_CENTER: &str = std_str!("PadCenter");
pub const STD_STR_FROM_CHAR: &str = std_str!("FromChar");
pub const STD_STR_CHAR_AT: &str = std_str!("CharAt");
pub const STD_STR_SET_CHAR_AT: &str = std_str!("SetCharAt");
pub const STD_STR_ORD: &str = std_str!("Ord");
pub const STD_STR_CHR: &str = std_str!("Chr");
pub const STD_STR_INSERT: &str = std_str!("Insert");
pub const STD_STR_DELETE: &str = std_str!("Delete");
pub const STD_STR_REVERSE: &str = std_str!("Reverse");
pub const STD_STR_TRIM_LEFT: &str = std_str!("TrimLeft");
pub const STD_STR_TRIM_RIGHT: &str = std_str!("TrimRight");
pub const STD_STR_LAST_INDEX_OF: &str = std_str!("LastIndexOf");
pub const STD_STR_FORMAT: &str = std_str!("Format");

pub const STD_CONV_INT_TO_STR: &str = std_conv!("IntToStr");
pub const STD_CONV_STR_TO_INT: &str = std_conv!("StrToInt");
pub const STD_CONV_REAL_TO_STR: &str = std_conv!("RealToStr");
pub const STD_CONV_STR_TO_REAL: &str = std_conv!("StrToReal");
pub const STD_CONV_CHAR_TO_STR: &str = std_conv!("CharToStr");
pub const STD_CONV_INT_TO_REAL: &str = std_conv!("IntToReal");
pub const STD_CONV_BOOL_TO_STR: &str = std_conv!("BoolToStr");
pub const STD_CONV_STR_TO_BOOL: &str = std_conv!("StrToBool");
pub const STD_CONV_INT_TO_HEX: &str = std_conv!("IntToHex");
pub const STD_CONV_HEX_TO_INT: &str = std_conv!("HexToInt");

pub const STD_MATH_PI: &str = std_math!("Pi");
pub const STD_MATH_SQRT: &str = std_math!("Sqrt");
pub const STD_MATH_POW: &str = std_math!("Pow");
pub const STD_MATH_FLOOR: &str = std_math!("Floor");
pub const STD_MATH_CEIL: &str = std_math!("Ceil");
pub const STD_MATH_ROUND: &str = std_math!("Round");
pub const STD_MATH_SIN: &str = std_math!("Sin");
pub const STD_MATH_COS: &str = std_math!("Cos");
pub const STD_MATH_LOG: &str = std_math!("Log");
pub const STD_MATH_ABS: &str = std_math!("Abs");
pub const STD_MATH_MIN: &str = std_math!("Min");
pub const STD_MATH_MAX: &str = std_math!("Max");
pub const STD_MATH_TAN: &str = std_math!("Tan");
pub const STD_MATH_ARC_SIN: &str = std_math!("ArcSin");
pub const STD_MATH_ARC_COS: &str = std_math!("ArcCos");
pub const STD_MATH_ARC_TAN: &str = std_math!("ArcTan");
pub const STD_MATH_ARC_TAN2: &str = std_math!("ArcTan2");
pub const STD_MATH_EXP: &str = std_math!("Exp");
pub const STD_MATH_LOG10: &str = std_math!("Log10");
pub const STD_MATH_LOG2: &str = std_math!("Log2");
pub const STD_MATH_TRUNC: &str = std_math!("Trunc");
pub const STD_MATH_FRAC: &str = std_math!("Frac");
pub const STD_MATH_SIGN: &str = std_math!("Sign");
pub const STD_MATH_CLAMP: &str = std_math!("Clamp");
pub const STD_MATH_RANDOM: &str = std_math!("Random");
pub const STD_MATH_RANDOM_INT: &str = std_math!("RandomInt");
pub const STD_MATH_RANDOMIZE: &str = std_math!("Randomize");

pub const STD_ARRAY_LENGTH: &str = std_array!("Length");
pub const STD_ARRAY_SORT: &str = std_array!("Sort");
pub const STD_ARRAY_REVERSE: &str = std_array!("Reverse");
pub const STD_ARRAY_CONTAINS: &str = std_array!("Contains");
pub const STD_ARRAY_INDEX_OF: &str = std_array!("IndexOf");
pub const STD_ARRAY_SLICE: &str = std_array!("Slice");
pub const STD_ARRAY_PUSH: &str = std_array!("Push");
pub const STD_ARRAY_POP: &str = std_array!("Pop");
pub const STD_ARRAY_MAP: &str = std_array!("Map");
pub const STD_ARRAY_FILTER: &str = std_array!("Filter");
pub const STD_ARRAY_REDUCE: &str = std_array!("Reduce");
pub const STD_ARRAY_CONCAT: &str = std_array!("Concat");
pub const STD_ARRAY_FILL: &str = std_array!("Fill");
pub const STD_ARRAY_FIND: &str = std_array!("Find");
pub const STD_ARRAY_FIND_INDEX: &str = std_array!("FindIndex");
pub const STD_ARRAY_ANY: &str = std_array!("Any");
pub const STD_ARRAY_ALL: &str = std_array!("All");
pub const STD_ARRAY_FLAT_MAP: &str = std_array!("FlatMap");
pub const STD_ARRAY_FOR_EACH: &str = std_array!("ForEach");

pub const STD_RESULT_UNWRAP: &str = std_result!("Unwrap");
pub const STD_RESULT_UNWRAP_OR: &str = std_result!("UnwrapOr");
pub const STD_RESULT_IS_OK: &str = std_result!("IsOk");
pub const STD_RESULT_IS_ERR: &str = std_result!("IsError");
pub const STD_RESULT_MAP: &str = std_result!("Map");
pub const STD_RESULT_AND_THEN: &str = std_result!("AndThen");
pub const STD_RESULT_OR_ELSE: &str = std_result!("OrElse");

pub const STD_OPTION_UNWRAP: &str = std_option!("Unwrap");
pub const STD_OPTION_UNWRAP_OR: &str = std_option!("UnwrapOr");
pub const STD_OPTION_IS_SOME: &str = std_option!("IsSome");
pub const STD_OPTION_IS_NONE: &str = std_option!("IsNone");
pub const STD_OPTION_MAP: &str = std_option!("Map");
pub const STD_OPTION_AND_THEN: &str = std_option!("AndThen");
pub const STD_OPTION_OR_ELSE: &str = std_option!("OrElse");

pub const STD_TASK_WAIT: &str = std_task!("Wait");
pub const STD_TASK_WAIT_ALL: &str = std_task!("WaitAll");

pub const STD_DICT_LENGTH: &str = std_dict!("Length");
pub const STD_DICT_CONTAINS_KEY: &str = std_dict!("ContainsKey");
pub const STD_DICT_KEYS: &str = std_dict!("Keys");
pub const STD_DICT_VALUES: &str = std_dict!("Values");
pub const STD_DICT_REMOVE: &str = std_dict!("Remove");
pub const STD_DICT_GET: &str = std_dict!("Get");
pub const STD_DICT_MERGE: &str = std_dict!("Merge");
pub const STD_DICT_MAP: &str = std_dict!("Map");
pub const STD_DICT_FILTER: &str = std_dict!("Filter");
