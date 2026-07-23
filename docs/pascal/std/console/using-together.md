# Using text and keyboard together

- Use **`Read` / `ReadLn`** for typed input and pipes (line discipline).
- Use **`ReadKey` / `ReadKeyEvent`** for games or immediate key handling.
- Use **`ReadEvent` / `PollEvent` / `ReadEventTimeout`** for TUI-style unified terminal events.
- Do not mix live `ReadKey*` and `ReadEvent*` in one loop: a live key is delivered to only one consumer.
- Do not assume that mixing `ReadKey` and `ReadKeyEvent` in one tight loop will interleave predictably without designing your loop; they are different subsystems.

## Example

```pascal
uses Std.Console;

begin
  Write('Name: ');
  var Name: string := ReadLn();
  WriteLn('Hello, ', Name);

  WriteLn('Press Escape or any printable key.');
  var Key: KeyEvent := ReadKeyEvent();
  if Key.kind = KeyKind.Escape then
    WriteLn('escape')
  else
    WriteLn(Key.ch)
end.
```

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
