# Format and characters

## `Text.IsNumeric(): boolean`

`true` if the string (after trim) parses as an **integer** or **real**, otherwise `false`.

```pascal
WriteLn('42'.IsNumeric());
WriteLn('nope'.IsNumeric());
```

---

## `Text.FromChar(N: integer): string`

Builds a string of `N` copies of the receiver. The receiver must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors. `N` ≤ 0 yields an empty string.

Positive `N` must be at most **1_000_000**. Larger counts raise a runtime error.

```pascal
WriteLn('─'.FromChar(40));
```

---

## `Text.CharAt(Index: integer): string`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

Indices count Unicode scalars. For a nonempty string, valid indices are
`0..S.Length()-1`. Empty strings have no valid character index. Invalid indices
report FP5021; the hint uses `S.Length()` for nonempty strings and recommends
checking `S.IsEmpty()` for empty strings.

```pascal
const C: string := 'Hello'.CharAt(0);
WriteLn(C);  // H
```

---

## `Text.SetCharAt(Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. `C` must contain exactly one Unicode scalar value. **Runtime error** if `Index` is out of bounds or `C` is empty or contains multiple characters.

`Index` follows the same scalar bounds and empty-string checks as `CharAt`.

```pascal
WriteLn('Hello'.SetCharAt(0, 'J'));  // Jello
```

---

## `Text.Ord(): integer`

Returns the Unicode codepoint (integer value) of the receiver. The receiver must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors.

```pascal
WriteLn('A'.Ord());  // 65
```

---

## `string.Chr(N: integer): string`

Returns the character with Unicode codepoint `N`. **Runtime error** if `N` is not a valid Unicode scalar value.

```pascal
WriteLn(string.Chr(65));  // A
```

---

## `Text.Format(Arguments...): string`

Returns a new string by substituting format specifiers in `Template` with the supplied arguments.

The string receiver is the template. Its heterogeneous arguments are positional only; named arguments are rejected.

```pascal
const Zoom: real := 2.0;
const CX: real := 0.5;
const CY: real := -0.5;
const Index: integer := 3;
const Name: string := 'sample';
const Status: string := 'Zoom: %fx Center: (%f, %f)'.Format(Zoom, CX, CY);
const Msg: string    := 'Item %d: %s'.Format(Index, Name);
const Pct: string    := '100%%'.Format();  // '100%'
```

### Specifiers

| Specifier | Accepted type | Example |
|-----------|--------------|---------|
| `%d` | `integer` | `'%d'.Format(42)` → `'42'` |
| `%f` | `real` or `integer` | `'%f'.Format(3.14)` → `'3.14'` |
| `%s` | `string` | `'%s'.Format('hi')` → `'hi'` |
| `%%` | *(no argument)* | `'100%%'.Format()` → `'100%'` |

`%f` accepts both `real` and `integer`. Integer arguments are rendered with at least one fractional digit: `'%f'.Format(42)` produces `'42.0'`.

### Runtime errors

- A trailing `%` at the end of `Template` is a runtime error.
- Unknown specifiers such as `%q` are a runtime error.
- Too few arguments, too many arguments, or a type mismatch for `%d`, `%f`, or `%s` are runtime errors.

---

## See also

- [Str overview](README.md)
- [`Std.Conv`](../../../std/text/conv.md)
- [Text index](../../../std/text/README.md)
