# `Std.Console`

Text output, line-buffered stdin, and terminal input (CRT-style screen control plus structured key and event APIs).

```pascal
program Example;
uses Std.Console;
begin
  WriteLn('ok')
end.
```

## Importing and names

After `uses Std.Console;` you can call symbols in either form:

| Style | Example |
|--------|---------|
| **Fully qualified** | `Std.Console.WriteLn('hi')` |
| **Short** | `WriteLn('hi')` |

Short names exist only for symbols that belong to a `uses`'d unit. If two imported units expose the **same** short name (for example `Length` from `Std.Str` and `Std.Array`), the compiler reports an **ambiguous** error at the use site; then use the full name (`Std.Str.Length`, `Std.Array.Length`).

Types follow the same idea: `KeyEvent` is the short form of `Std.Console.KeyEvent` when `Std.Console` is imported.

## Quick reference

Everything below requires `uses Std.Console;`.

| Kind | Name | Notes |
|------|------|--------|
| procedure | `Write(...)` | variadic |
| procedure | `WriteLn(...)` | variadic |
| procedure | `ClrScr()` | clear only the active console window |
| procedure | `ClrEol()` | clear from cursor to the right edge of the active window |
| procedure | `GotoXY(X, Y)` | 1-based coordinates inside the active window |
| function | `WhereX(): integer` | current 1-based cursor column inside the active window |
| function | `WhereY(): integer` | current 1-based cursor row inside the active window |
| function | `WindMin(): integer` | packed upper-left corner of the active window (low byte `X`, high byte `Y`) |
| function | `WindMax(): integer` | packed lower-right corner of the active window (low byte `X`, high byte `Y`) |
| procedure | `DelLine()` | delete line at cursor row inside active window |
| procedure | `InsLine()` | insert blank line at cursor row inside active window |
| procedure | `Window(X1, Y1, X2, Y2)` | set the active console window (screen-relative) |
| procedure | `TextColor(Color)` | set foreground color for subsequent writes |
| procedure | `TextBackground(Color)` | set background color for subsequent writes |
| procedure | `TextColorRGB(R, G, B)` | set fg to 24-bit truecolor (0–255 per channel), outside packed `TextAttr` state |
| procedure | `TextBackgroundRGB(R, G, B)` | set bg to 24-bit truecolor (0–255 per channel), outside packed `TextAttr` state |
| procedure | `TextColor256(Index)` | set fg to 256-color palette index (0–255), outside packed `TextAttr` state |
| procedure | `TextBackground256(Index)` | set bg to 256-color palette index (0–255), outside packed `TextAttr` state |
| procedure | `HighVideo()` | set bright foreground intensity bit |
| procedure | `LowVideo()` | clear bright foreground intensity bit |
| procedure | `NormVideo()` | reset attributes to light-gray on black |
| function | `TextAttr(): integer` | packed text attribute (`Background * 16 + Foreground`) |
| procedure | `SetTextAttr(Attr)` | set packed text attribute `0..255` |
| procedure | `Delay(Milliseconds)` | sleep for a non-negative integer number of milliseconds |
| procedure | `CursorOn()` | show the terminal cursor |
| procedure | `CursorOff()` | hide the terminal cursor |
| procedure | `CursorBig()` | show cursor using block style |
| procedure | `TextMode(Mode)` | reset CRT state and store the mode value |
| function | `LastMode(): integer` | last value passed to `TextMode` |
| function | `ScreenWidth(): integer` | current console screen width |
| function | `ScreenHeight(): integer` | current console screen height |
| procedure | `Sound(Hz)` | emit one terminal bell for positive `Hz` |
| procedure | `NoSound()` | stop active tone state (no-op) |
| procedure | `AssignCrt()` | enable CRT mode |
| function | `ReadLn(): string` | line input |
| function | `Read(): char` | same buffer as `ReadLn` |
| function | `ReadKey(): char` | key-by-key, separate from `ReadKeyEvent` |
| function | `KeyPressed(): boolean` | true if `ReadKey` or `ReadKeyEvent` has data waiting |
| function | `ReadKeyEvent(): KeyEvent` | structured key + modifiers |
| function | `EventPending(): boolean` | true if `ReadEvent()` has data waiting |
| function | `ReadEvent(): Event` | unified terminal event for keyboard, mouse, resize, paste, and focus |
| function | `ReadEventTimeout(Milliseconds: integer): Option of Event` | wait up to N ms for an event; requires `EnableRawMode()` first |
| function | `PollEvent(): Option of Event` | non-blocking check; `None` if no event is ready; requires `EnableRawMode()` first |
| procedure | `EnableRawMode()` | explicitly enable terminal raw mode |
| procedure | `DisableRawMode()` | explicitly disable terminal raw mode |
| procedure | `EnterAltScreen()` | switch to the alternate terminal screen |
| procedure | `LeaveAltScreen()` | leave the alternate terminal screen |
| procedure | `EnableMouse()` | enable mouse reporting |
| procedure | `DisableMouse()` | disable mouse reporting |
| procedure | `EnableFocus()` | enable focus gained/lost reporting |
| procedure | `DisableFocus()` | disable focus reporting |
| procedure | `EnablePaste()` | enable bracketed paste reporting |
| procedure | `DisablePaste()` | disable bracketed paste reporting |
| type | `KeyEvent` | record |
| type | `KeyKind` | enum |
| type | `Event` | record |
| type | `EventKind` | enum |
| type | `MouseAction` | enum |
| type | `MouseButton` | enum |
| const | `Black`, `Blue`, `Green`, …, `White` | CRT-style color indices `0..15` |
| const | `Blink` | text-attribute blink bit (`128`) |
| const | `BW40`, `C40`, `BW80`, `C80`, `CO40`, `CO80`, `Mono`, `Font8x8` | text-mode compatibility constants |

Extended color procedures (`TextColorRGB`, `TextBackgroundRGB`, `TextColor256`, `TextBackground256`) send terminal ANSI color escapes directly. They do not update the packed 16-color CRT attribute returned by `TextAttr()`. Calling `TextColor`, `TextBackground`, `HighVideo`, `LowVideo`, `NormVideo`, or `SetTextAttr` afterwards switches back to the packed CRT attribute path and overrides the extended color.

## Topics

| Topic | Description |
|-------|-------------|
| [Output](output.md) | `Write`, `WriteLn` |
| [Screen control](screen.md) | Windows, cursor, scrolling |
| [Screen utilities](screen-misc.md) | `Delay`, cursor visibility, `TextMode`, bell |
| [Colors and attributes](colors.md) | CRT palette, RGB, 256-color |
| [Types](types.md) | `KeyEvent`, `Event`, enums |
| [Line input](input.md) | `ReadLn`, `Read` |
| [Keyboard](keyboard.md) | `ReadKey`, `ReadKeyEvent`, `KeyPressed` |
| [Terminal events](events.md) | `ReadEvent`, raw mode, alt screen |
| [Using together](using-together.md) | Mixing line and key input |

## See also

- [`Std.Tui`](../tui/session.md) — hosted terminal applications
- [Standard library index](../README.md)

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| User-facing registration | [`loaded/console.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/console.rs) |
| Console backend | [`console/mod.rs`](../../../../crates/fpas-std/src/console/mod.rs) |
| Key and event types | [`key_event.rs`](../../../../crates/fpas-std/src/key_event.rs), [`console_event.rs`](../../../../crates/fpas-std/src/console_event.rs) |
| Bytecode / VM | [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs), [`vm/mod.rs`](../../../../crates/fpas-vm/src/vm/mod.rs) |
| Code generation | [`std_calls/console/`](../../../../crates/fpas-compiler/src/compiler/std_calls/console/mod.rs) |
