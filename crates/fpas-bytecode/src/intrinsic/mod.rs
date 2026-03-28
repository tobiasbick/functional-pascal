//! Built-in intrinsic identifiers (embedded in `Op::Intrinsic`).
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root); each `Std.*` unit page maps API names to these variants.
//! **Maintenance:** When adding or renumbering variants, update that documentation and the affected implementation crates.

mod decode;

/// VM intrinsic opcode payload (`Op::Intrinsic(self as u16)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Intrinsic {
    ConsoleReadLn = 1,
    ConsoleRead = 2,
    ConsoleReadKey = 3,
    ConsoleKeyPressed = 4,
    ConsoleReadKeyEvent = 5,
    ConsoleClrScr = 6,
    ConsoleClrEol = 7,
    ConsoleGotoXY = 8,
    ConsoleWhereX = 9,
    ConsoleWhereY = 10,
    ConsoleWindMin = 11,
    ConsoleWindMax = 12,
    ConsoleWindow = 13,
    ConsoleTextColor = 14,
    ConsoleTextBackground = 15,
    ConsoleDelay = 16,
    ConsoleCursorOn = 17,
    ConsoleCursorOff = 18,
    ConsoleDelLine = 170,
    ConsoleInsLine = 171,
    ConsoleHighVideo = 172,
    ConsoleLowVideo = 173,
    ConsoleNormVideo = 174,
    ConsoleTextAttr = 175,
    ConsoleSetTextAttr = 176,
    ConsoleCursorBig = 177,
    ConsoleTextMode = 178,
    ConsoleLastMode = 179,
    ConsoleScreenWidth = 180,
    ConsoleScreenHeight = 181,
    ConsoleSound = 182,
    ConsoleNoSound = 183,
    ConsoleAssignCrt = 184,
    ConsoleEventPending = 185,
    ConsoleReadEvent = 186,
    ConsoleEnableRawMode = 187,
    ConsoleDisableRawMode = 188,
    ConsoleEnterAltScreen = 189,
    ConsoleLeaveAltScreen = 190,
    ConsoleEnableMouse = 191,
    ConsoleDisableMouse = 192,
    ConsoleEnableFocus = 193,
    ConsoleDisableFocus = 194,
    ConsoleEnablePaste = 195,
    ConsoleDisablePaste = 196,

    StrLength = 20,
    StrToUpper = 21,
    StrToLower = 22,
    StrTrim = 23,
    StrContains = 24,
    StrStartsWith = 25,
    StrEndsWith = 26,
    StrSubstring = 27,
    StrIndexOf = 28,
    StrReplace = 29,
    StrSplit = 30,
    StrJoin = 31,
    StrIsNumeric = 32,

    ConvIntToStr = 40,
    ConvStrToInt = 41,
    ConvRealToStr = 42,
    ConvStrToReal = 43,
    ConvCharToStr = 44,
    ConvIntToReal = 45,

    MathSqrt = 60,
    MathPow = 61,
    MathFloor = 62,
    MathCeil = 63,
    MathRound = 64,
    MathSin = 65,
    MathCos = 66,
    MathLog = 67,
    MathMin = 68,
    MathMax = 69,
    MathAbs = 70,

    ArrayLength = 80,
    ArraySort = 81,
    ArrayReverse = 82,
    ArrayContains = 83,
    ArrayIndexOf = 84,
    ArraySlice = 85,
    ArrayMap = 86,
    ArrayFilter = 87,
    ArrayReduce = 88,

    ResultUnwrap = 90,
    ResultUnwrapOr = 91,
    ResultIsOk = 92,
    ResultIsError = 93,
    /// `Std.Result.Map(R, F)` — `Ok(v)` → `Ok(F(v))`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result.md`
    ResultMap = 130,
    /// `Std.Result.AndThen(R, F)` — `Ok(v)` → `F(v)`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result.md`
    ResultAndThen = 131,
    /// `Std.Result.OrElse(R, F)` — `Ok(v)` passthrough, `Error(e)` → `F(e)`.
    ///
    /// **Documentation:** `docs/pascal/std/result.md`
    ResultOrElse = 132,
    OptionUnwrap = 94,
    OptionUnwrapOr = 95,
    OptionIsSome = 96,
    OptionIsNone = 97,
    /// `Std.Option.Map(O, F)` — `Some(v)` → `Some(F(v))`, `None` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/option.md`
    OptionMap = 133,
    /// `Std.Option.AndThen(O, F)` — `Some(v)` → `F(v)`, `None` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/option.md`
    OptionAndThen = 134,
    /// `Std.Option.OrElse(O, F)` — `Some(v)` passthrough, `None` → `F()`.
    ///
    /// **Documentation:** `docs/pascal/std/option.md`
    OptionOrElse = 135,

    /// Create unbuffered channel. Pushes `Value::Channel(id)`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelMake = 100,
    /// Create buffered channel. Pops `size: integer`, pushes `Value::Channel(id)`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelMakeBuffered = 101,
    /// Send value on channel. Pops `value`, pops `channel`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelSend = 102,
    /// Receive value from channel (blocking). Pops `channel`, pushes received value.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelRecv = 103,
    /// Non-blocking receive. Pops `channel`, pushes `OptionSome(value)` or `OptionNone`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelTryRecv = 104,
    /// Close a channel. Pops `channel`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    ChannelClose = 105,
    /// Wait for a task to complete. Pops `task`, pushes its return value.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    TaskWait = 110,
    /// Wait for all tasks to complete. Pops `array of task`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    TaskWaitAll = 111,

    /// **Documentation:** `docs/future/advanced-types.md`
    DictLength = 120,
    /// **Documentation:** `docs/future/advanced-types.md`
    DictContainsKey = 121,
    /// **Documentation:** `docs/future/advanced-types.md`
    DictKeys = 122,
    /// **Documentation:** `docs/future/advanced-types.md`
    DictValues = 123,
    /// **Documentation:** `docs/future/advanced-types.md`
    DictRemove = 124,
}

impl From<Intrinsic> for u16 {
    fn from(intrinsic: Intrinsic) -> Self {
        intrinsic as u16
    }
}
