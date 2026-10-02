# Search

## `function Contains(S: string; Sub: string): boolean`

`true` if `Sub` occurs anywhere in `S`, else `false`.

```pascal
uses Std.Console as Console;
uses Std.Str as Str;

if Str.Contains('abc', 'b') then
  Console.WriteLn('yes');
end if;
```

---

## `function StartsWith(S: string; Pre: string): boolean`

`true` if `S` begins with `Pre`.

```pascal
uses Std.Console as Console;

Console.WriteLn(StartsWith('abc', 'ab'));
```

---

## `function EndsWith(S: string; Suf: string): boolean`

`true` if `S` ends with `Suf`.

```pascal
uses Std.Console as Console;

Console.WriteLn(EndsWith('abc', 'bc'));
```

---

## `function Substring(S: string; Start: integer; Len: integer): string`

Copies `Len` characters starting at `Start`. **Bounds are checked at runtime**; invalid ranges produce a runtime error.

```pascal
uses Std.Console as Console;

Console.WriteLn(Substring('Hello', 0, 3));
```

---

## `function IndexOf(S: string; Sub: string): integer`

Returns the **first** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
uses Std.Console as Console;

Console.WriteLn(IndexOf('aba', 'a'));
Console.WriteLn(IndexOf('aba', 'z'));
```

---

## `function LastIndexOf(S: string; Sub: string): integer`

Returns the **last** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
uses Std.Console as Console;

Console.WriteLn(LastIndexOf('abcabc', 'abc'));  // 3
Console.WriteLn(LastIndexOf('abc', 'z'));       // -1
```

## See also

- [Str overview](README.md)
- [Split and join](split-join.md)
- [Text index](../README.md)
