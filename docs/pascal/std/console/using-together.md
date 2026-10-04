# Using text and keyboard together

- Use **`ReadText` / `ReadLn`** for typed input and pipes (line discipline).
- Use **`ReadKey` / `ReadKeyEvent`** for games or immediate key handling.
- Use **`ReadEvent` / `PollEvent` / `ReadEventTimeout`** for TUI-style unified terminal events.
- Do not mix live `ReadKey*` and `ReadEvent*` in one loop: a live key is delivered to only one consumer.
- Do not assume that mixing `ReadKey` and `ReadKeyEvent` in one tight loop will interleave predictably without designing your loop; they are different subsystems.

## Example

```pascal
program Example;

uses Std.Console as Console;

begin
  Console.WriteText('Name: ');
  const Name: string := Console.ReadLn();
  Console.WriteLn('Hello, ', Name);

  Console.WriteLn('Press Escape or any printable key.');
  const Key: Console.KeyEvent := Console.ReadKeyEvent();
  if Key.kind = Console.KeyKind.Escape then
    Console.WriteLn('escape');
  else
    Console.WriteLn(Key.ch); end if;
end program;
```

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
