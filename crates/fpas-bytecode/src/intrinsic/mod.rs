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
    /// `Std.Console.ReadEventTimeout(Ms)` — wait up to `Ms` milliseconds; returns `option of Event`.
    ///
    /// **Documentation:** `docs/pascal/std/console.md`
    ConsoleReadEventTimeout = 197,
    /// `Std.Console.PollEvent()` — non-blocking; returns `option of Event`.
    ///
    /// **Documentation:** `docs/pascal/std/console.md`
    ConsolePollEvent = 198,

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
    /// **Documentation:** `docs/pascal/std/dict.md`
    DictGet = 125,
    /// **Documentation:** `docs/pascal/std/dict.md`
    DictMerge = 126,
    /// `Std.Dict.Map(D, F)` — transform every value; `F: function(V): V2`.
    ///
    /// **Documentation:** `docs/pascal/std/dict.md`
    DictMap = 127,
    /// `Std.Dict.Filter(D, F)` — keep entries where `F(K, V)` is true.
    ///
    /// **Documentation:** `docs/pascal/std/dict.md`
    DictFilter = 128,

    /// `Std.Str.Repeat(S, N)` — repeat string N times.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrRepeat = 200,
    /// `Std.Str.PadLeft(S, Width, PadChar)` — left-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrPadLeft = 201,
    /// `Std.Str.PadRight(S, Width, PadChar)` — right-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrPadRight = 202,
    /// `Std.Str.PadCenter(S, Width, PadChar)` — center-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrPadCenter = 203,
    /// `Std.Str.FromChar(C, N)` — create string of N copies of char C.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrFromChar = 204,
    /// `Std.Str.CharAt(S, Index)` — character at zero-based index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrCharAt = 205,
    /// `Std.Str.SetCharAt(S, Index, C)` — return new string with char replaced.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrSetCharAt = 206,
    /// `Std.Str.Ord(C)` — Unicode code point of a char.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrOrd = 207,
    /// `Std.Str.Chr(N)` — char from Unicode code point.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrChr = 208,
    /// `Std.Str.Insert(S, Index, Sub)` — insert substring at index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrInsert = 209,
    /// `Std.Str.Delete(S, Index, Len)` — delete Len chars starting at Index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrDelete = 210,
    /// `Std.Str.Reverse(S)` — reverse a string.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrReverse = 211,
    /// `Std.Str.TrimLeft(S)` — trim leading whitespace.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrTrimLeft = 212,
    /// `Std.Str.TrimRight(S)` — trim trailing whitespace.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrTrimRight = 213,
    /// `Std.Str.LastIndexOf(S, Sub)` — last occurrence index or -1.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    StrLastIndexOf = 214,

    /// `Std.Conv.BoolToStr(B)` — boolean to `'true'`/`'false'`.
    ///
    /// **Documentation:** `docs/pascal/std/conv.md`
    ConvBoolToStr = 215,
    /// `Std.Conv.StrToBool(S)` — parse `'true'`/`'false'` to boolean.
    ///
    /// **Documentation:** `docs/pascal/std/conv.md`
    ConvStrToBool = 216,
    /// `Std.Conv.IntToHex(N)` — integer to hexadecimal string.
    ///
    /// **Documentation:** `docs/pascal/std/conv.md`
    ConvIntToHex = 217,
    /// `Std.Conv.HexToInt(S)` — hexadecimal string to integer.
    ///
    /// **Documentation:** `docs/pascal/std/conv.md`
    ConvHexToInt = 218,

    /// `Std.Math.Tan(R)` — tangent.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathTan = 219,
    /// `Std.Math.ArcSin(R)` — arcsine.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathArcSin = 220,
    /// `Std.Math.ArcCos(R)` — arccosine.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathArcCos = 221,
    /// `Std.Math.ArcTan(R)` — arctangent.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathArcTan = 222,
    /// `Std.Math.ArcTan2(Y, X)` — two-argument arctangent.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathArcTan2 = 223,
    /// `Std.Math.Exp(R)` — e^R.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathExp = 224,
    /// `Std.Math.Log10(R)` — base-10 logarithm.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathLog10 = 225,
    /// `Std.Math.Log2(R)` — base-2 logarithm.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathLog2 = 226,
    /// `Std.Math.Trunc(R)` — truncate toward zero, return integer.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathTrunc = 227,
    /// `Std.Math.Frac(R)` — fractional part.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathFrac = 228,
    /// `Std.Math.Sign(X)` — sign (-1, 0, 1), polymorphic integer/real.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathSign = 229,
    /// `Std.Math.Clamp(X, Lo, Hi)` — clamp to range, polymorphic.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathClamp = 230,
    /// `Std.Math.Random` — random real in [0, 1).
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathRandom = 231,
    /// `Std.Math.RandomInt(N)` — random integer in [0, N).
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathRandomInt = 232,
    /// `Std.Math.Randomize` — seed the RNG.
    ///
    /// **Documentation:** `docs/pascal/std/math.md`
    MathRandomize = 233,

    /// `Std.Array.Concat(A, B)` — concatenate two arrays.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayConcat = 234,
    /// `Std.Array.Fill(Value, Count)` — create array of Count copies of Value.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayFill = 235,
    /// `Std.Array.Find(Arr, Pred)` — first element matching predicate, or None.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayFind = 236,
    /// `Std.Array.FindIndex(Arr, Pred)` — index of first match, or -1.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayFindIndex = 237,
    /// `Std.Array.Any(Arr, Pred)` — true if any element matches.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayAny = 238,
    /// `Std.Array.All(Arr, Pred)` — true if all elements match.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayAll = 239,
    /// `Std.Array.FlatMap(Arr, F)` — map then flatten.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayFlatMap = 240,
    /// `Std.Array.ForEach(Arr, F)` — apply F to each element (returns unit).
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    ArrayForEach = 241,
}

impl From<Intrinsic> for u16 {
    fn from(intrinsic: Intrinsic) -> Self {
        intrinsic as Self
    }
}
