# Future: Standard Library Extensions

> Planned. Extends existing standard library units with additional functions and iteration support.

## Std.Str — String Helpers

### Motivation

Building formatted text output requires padding, repeating characters, and character-level access. The current `Std.Str` surface ([str.md](../pascal/std/str.md)) lacks these operations.

### Proposed Additions

| Kind | Signature | Description |
|------|-----------|-------------|
| function | `RepeatStr(S: string; Count: integer): string` | Repeat `S` exactly `Count` times |
| function | `PadLeft(S: string; Width: integer; Fill: char): string` | Left-pad to `Width` with `Fill` |
| function | `PadRight(S: string; Width: integer; Fill: char): string` | Right-pad to `Width` with `Fill` |
| function | `PadCenter(S: string; Width: integer; Fill: char): string` | Center-pad to `Width` with `Fill` |
| function | `CharAt(S: string; Index: integer): char` | Character at 0-based index |
| function | `SetCharAt(S: string; Index: integer; C: char): string` | New string with one character replaced |
| function | `FromChar(C: char; Count: integer): string` | Build string from repeated char |

### Examples

```pascal
uses Std.Str;

var Line: string := RepeatStr('─', 40);
var Header: string := PadCenter('Title', 40, ' ');
var C: char := CharAt('Hello', 0);   { 'H' }
```

---

## Std.Str — String Indexing

### Motivation

Accessing a single character in a string currently requires `Substring(S, I, 1)`, which returns a `string`, not a `char`. Direct index syntax would be more natural.

### Proposed Syntax

```pascal
var S: string := 'Hello';
var C: char := S[0];            { 'H' — 0-based character index }
```

Index access on `string` yields `char`. Out-of-bounds is a runtime error.

---

## Std.Console — Timer and Timeout Events

### Motivation

Interactive programs need idle processing, cursor blinking, and periodic updates. `ReadEvent()` currently blocks indefinitely — there is no way to specify a timeout or receive periodic timer events.

### Proposed Additions

| Kind | Signature | Description |
|------|-----------|-------------|
| function | `ReadEventTimeout(Milliseconds: integer): Option of Event` | Wait up to `Milliseconds` ms; return `None` on timeout |
| function | `PollEvent(): Option of Event` | Non-blocking; return `None` if no event pending |

### Examples

```pascal
uses Std.Console, Std.Option;

{ Wait up to 100ms for an event, then do idle work }
var E: Option of Event := ReadEventTimeout(100);
case E of
  Some(Ev): HandleEvent(Ev);
  None: IdleUpdate();
end;
```

`EventPending()` already exists for key/mouse polling, but `PollEvent` provides the full `Event` record without blocking.

---

## For-In over Dict

### Motivation

Dictionaries (`dict of K to V`) cannot be iterated with `for-in`. The only way to iterate is `Keys(D)` then index lookup, which is indirect and less efficient.

### Proposed Syntax

```pascal
uses Std.Dict;

var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];

for Key: string in Ages do
  WriteLn(Key + ': ' + IntToStr(Ages[Key]));
```

Iteration yields keys in insertion order (matching `Keys()` behavior). A key-value form could also be considered:

```pascal
for Key: string, Value: integer in Ages do
  WriteLn(Key + ': ' + IntToStr(Value));
```

---

## Std.Dict — Index Access

### Motivation

Dict values are currently accessed only through procedural functions. Index syntax would be more natural:

```pascal
var Age: integer := Ages['Alice'];         { instead of a Get function }
```

Out-of-bounds (missing key) would be a runtime error, or return an `Option` depending on design choice.

---

## Docs and Specs to Extend

- [std/str.md](../pascal/std/str.md): `RepeatStr`, `PadLeft`, `PadRight`, `PadCenter`, `CharAt`, `SetCharAt`, `FromChar`
- [std/console.md](../pascal/std/console.md): `ReadEventTimeout`, `PollEvent`
- [02-basics.md](../pascal/02-basics.md): string index access `S[I]`
- [03-control-flow.md](../pascal/03-control-flow.md): `for-in` over `dict`
- [std/dict.md](../pascal/std/dict.md): index access syntax, key-value iteration
- [grammar.ebnf](../specs/grammar.ebnf): `ForInDict`, `DictIndexExpr`, `StringIndexExpr` productions
