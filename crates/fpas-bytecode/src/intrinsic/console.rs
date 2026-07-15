//! `Std.Console` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/console/README.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Console.*`.
///
/// **Documentation:** `docs/pascal/std/console/README.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ConsoleIntrinsic {
    ReadLn = 1,
    Read = 2,
    ReadKey = 3,
    KeyPressed = 4,
    ReadKeyEvent = 5,
    ClrScr = 6,
    ClrEol = 7,
    GotoXY = 8,
    WhereX = 9,
    WhereY = 10,
    WindMin = 11,
    WindMax = 12,
    Window = 13,
    TextColor = 14,
    TextBackground = 15,
    Delay = 16,
    CursorOn = 17,
    CursorOff = 18,
    DelLine = 170,
    InsLine = 171,
    HighVideo = 172,
    LowVideo = 173,
    NormVideo = 174,
    TextAttr = 175,
    SetTextAttr = 176,
    CursorBig = 177,
    TextMode = 178,
    LastMode = 179,
    ScreenWidth = 180,
    ScreenHeight = 181,
    Sound = 182,
    NoSound = 183,
    AssignCrt = 184,
    EventPending = 185,
    ReadEvent = 186,
    EnableRawMode = 187,
    DisableRawMode = 188,
    EnterAltScreen = 189,
    LeaveAltScreen = 190,
    EnableMouse = 191,
    DisableMouse = 192,
    EnableFocus = 193,
    DisableFocus = 194,
    EnablePaste = 195,
    DisablePaste = 196,
    /// `Std.Console.ReadEventTimeout(Ms)` — wait up to `Ms` milliseconds; returns `option of Event`.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    ReadEventTimeout = 197,
    /// `Std.Console.PollEvent()` — non-blocking; returns `option of Event`.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    PollEvent = 198,
    /// `Std.Console.TextColorRGB(R, G, B)` — set fg to 24-bit truecolor.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    TextColorRGB = 243,
    /// `Std.Console.TextBackgroundRGB(R, G, B)` — set bg to 24-bit truecolor.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    TextBackgroundRGB = 244,
    /// `Std.Console.TextColor256(Index)` — set fg to 256-color palette index.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    TextColor256 = 245,
    /// `Std.Console.TextBackground256(Index)` — set bg to 256-color palette index.
    ///
    /// **Documentation:** `docs/pascal/std/console/README.md`
    TextBackground256 = 246,
    /// Constructs a classic CRT palette `Std.Console.Color`.
    CrtColor = 381,
    /// Constructs an ANSI 256-palette `Std.Console.Color`.
    Ansi256Color = 382,
    /// Constructs a truecolor `Std.Console.Color`.
    RgbColor = 383,
    /// Starts or nests a deferred console frame.
    BeginFrame = 384,
    /// Presents the current console frame.
    Present = 385,
    /// Paints one `Std.Console.Cell`.
    PutCell = 386,
    /// Reads one `Std.Console.Cell`.
    GetCell = 387,
    /// Fills a `Std.Console.Rect` with one cell.
    FillRect = 388,
    /// Paints an array of cells from left to right.
    WriteCells = 389,
    /// Saves a rectangular console region.
    SaveRegion = 390,
    /// Restores and consumes a saved console region.
    RestoreRegion = 391,
    /// Discards and consumes a saved console region.
    DiscardRegion = 392,
    /// Measures a string in terminal columns.
    DisplayWidth = 393,
}
