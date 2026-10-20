# Line input

`ReadLn` and `ReadText` consume the same line-buffered text stream. Both return
`Option of string`. `Some` contains input and `None` signals end of input.
Reading a terminal can block until a line is available.

## ReadLn

`function ReadLn(): Option of string`

Reads the remaining text of the next line without its terminator. An empty line
is `Some('')`. A final line without a line terminator is returned once, followed
by `None`. Further reads at EOF return `None`.

This loop copies all input lines, including empty lines, and terminates at EOF:

```pascal id=example-1 fragment context=body-console
var Reading := true;
while Reading do
  case Console.ReadLn() of
    when Some(const Line):
      Console.WriteLn(Line);
    when None:
      Reading := false;
  end case;
end while;
```

## ReadText

`function ReadText(): Option of string`

Reads one Unicode scalar from the current line buffer, refilling it if needed.
LF and CRLF input terminators appear as `Some(#10)`. A final unterminated line
has no added newline. A lone CR remains input. Once the stream is drained,
`ReadText` returns `None`.

```pascal id=example-2 fragment context=body-console
case Console.ReadText() of
  when Some(const Character):
    Console.WriteText(Character);
  when None:
    Console.WriteLn('End of input');
end case;
```

Both functions consume the shared buffer in order. Reading its final character
before a line terminator leaves `ReadLn()` to return `Some('')`; consuming the
final character of an unterminated line leaves it to return `None`.

## Errors and scripted input

Host read failures, including invalid UTF-8, remain runtime input diagnostics.
They do not return `None`.

`Test.PushReadLn` and scripted `readln` events queue complete logical lines,
including their line terminator. An empty queue in an open debugger input
channel remains a missing-input diagnostic. Explicit debugger EOF closes the
stream; pending characters and queued lines are consumed before `None` is
returned. Debugger input remains separate from protocol stdin.

See the [Console overview](README.md), [test runner](../testing/test.md) and
[debugger input channel](../../tools/debugger.md).
