# Format and characters

## `function IsNumeric(S: string): boolean`

`true` if the string (after trim) parses as an **integer** or **real**, otherwise `false`.

```pascal
WriteLn(IsNumeric('42'));
WriteLn(IsNumeric('nope'))
```

---

## `function FromChar(C: string; Count: integer): string`

Builds a string of `Count` copies of `C`. `Count` ≤ 0 yields an empty string.

```pascal
WriteLn(FromChar('─', 40))
```

---

## `function CharAt(S: string; Index: integer): string`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

```pascal
var C: string := CharAt('Hello', 0);
WriteLn(C)  { H }
```

---

## `function SetCharAt(S: string; Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. **Runtime error** if out of bounds.

```pascal
WriteLn(SetCharAt('Hello', 0, 'J'))  { Jello }
```

---

## `function Ord(C: string): integer`

Returns the Unicode codepoint (integer value) of `C`.

```pascal
WriteLn(Ord('A'))  { 65 }
```

---

## `function Chr(N: integer): string`

Returns the character with Unicode codepoint `N`. **Runtime error** if `N` is not a valid Unicode scalar value.

```pascal
WriteLn(Chr(65))  { A }
```

---

## `function Format(Template: string; ...): string`

Returns a new string by substituting format specifiers in `Template` with the supplied arguments.

`Template` is mandatory. `Format()` without a template string is a compile error.

```pascal
var Status: string := Format('Zoom: %fx Center: (%f, %f)', Zoom, CX, CY);
var Msg: string    := Format('Item %d: %s', Index, Name);
var Pct: string    := Format('100%%');  { '100%' }
```

### Specifiers

| Specifier | Accepted type | Example |
|-----------|--------------|---------|
| `%d` | `integer` | `Format('%d', 42)` → `'42'` |
| `%f` | `real` or `integer` | `Format('%f', 3.14)` → `'3.14'` |
| `%s` | `string` or `string` | `Format('%s', 'hi')` → `'hi'` |
| `%%` | *(no argument)* | `Format('100%%')` → `'100%'` |

`%f` accepts both `real` and `integer`. Integer arguments are rendered with at least one fractional digit: `Format('%f', 42)` produces `'42.0'`.

### Runtime errors

- A trailing `%` at the end of `Template` is a runtime error.
- Unknown specifiers such as `%q` are a runtime error.
- Too few arguments, too many arguments, or a type mismatch for `%d`, `%f`, or `%s` are runtime errors.

---

## See also

- [Str overview](README.md)
- [`Std.Conv`](../conv.md)
- [Text index](../README.md)
