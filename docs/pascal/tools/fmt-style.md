# Formatter style rules

Canonical output rules for the AST pretty-printer. These are **normative for `fpas fmt`** once implemented. The emitter encodes them; this file is the human-readable spec.

**Status:** **complete** (2026-06). Normative for [`fpas fmt`](../../../crates/fpas-cli/src/cli_fmt/); invoke manually — no watch/LSP. Edit golden examples when the style changes; the emitter must match them.

**How to read this file**

| Section | What the code blocks show |
|---------|---------------------------|
| [Formatted output](#formatted-output-fpas-fmt) | **After `fpas fmt`** — complete `.fpas` files (golden output). This is what the formatter must produce. |
| [More examples — snippets](#more-examples--record-types-snippet) | **After `fpas fmt`** — same rules, but only a `type` slice (not a full file). |
| Rules below (indent, semicolons, …) | Textual spec; if a rule disagrees with a golden example, **fix the example or the rule**, then implement. |

There is **no** “messy input” column in this doc yet. Source before formatting may omit `begin` / `end`, use `WRITELN`, or extra blank lines — those are normalized in the golden blocks. Comments are preserved (see [Comments](#comments)).

Spec links: [`language/basics/README.md`](../language/basics/README.md), [`language/control-flow/README.md`](../language/control-flow/README.md), [`.cursor/rules/functional-pascal.mdc`](../../../.cursor/rules/functional-pascal.mdc).

---

## Formatted output (`fpas fmt`)

**Golden output.** Each block below is a **complete file** as written to disk after `fpas fmt` — including every `begin` / `end` the formatter inserts.

Visual checklist:

- `program` / `unit` header → **one blank line** → `uses` (if any) → **one blank line** → rest
- Every **program** ends with `begin` … `end.` (period on `end`)
- Every **function** / **procedure** / **method** body: `begin` … `end`
- Every **`if` / `else`**, **`for` / `while`**, **`case` arm**: extra nested `begin` … `end` (even for a single statement)
- **`repeat` … `until`**: no extra `begin` / `end` around the body
- **No** spare blank lines inside `begin` … `end` blocks

### Program — minimal

```pascal
program Hello;

begin
  WriteLn('Hello, World!')
end.
```

### Program — with `uses`

```pascal
program Hello;

uses Std.Console;

begin
  WriteLn('Hello, World!')
end.
```

### Program — control flow (`if`, `case`, `for`, `while`, `repeat`)

Same file **after** fmt (input might use `if x then WriteLn(...)` without branch blocks; output does not):

```pascal
program ControlFlowDemo;

uses Std.Console, Std.Conv;

begin
  var X: integer := 5;
  if X > 0 then
  begin
    WriteLn('positive')
  end
  else if X = 0 then
  begin
    WriteLn('zero')
  end
  else
  begin
    WriteLn('negative')
  end;

  case X of
    1:
    begin
      WriteLn('one')
    end;
    2, 3:
    begin
      WriteLn('two or three')
    end;
    10..20:
    begin
      WriteLn('ten to twenty')
    end
  else
  begin
    WriteLn('other')
  end
  end;

  for I: integer := 1 to 3 do
  begin
    WriteLn(IntToStr(I))
  end;

  while X < 10 do
  begin
    X := X + 1
  end;

  mutable var N: integer := 0;
  repeat
    WriteLn(IntToStr(N));
    N := N + 1
  until N >= 3
end.
```

### Program — `type` + record methods + `begin` body

Golden output for a file like [`examples/pascal/record-methods/point.fpas`](../../../examples/pascal/record-methods/point.fpas) (comments, extra blank lines, and missing header blank lines from the repo copy are **not** in the output).

```pascal
program PointExample;

uses Std.Console, Std.Conv;

type
  Point = record
    X: integer;
    Y: integer;

    function Sum(Self: Point): integer;
    begin
      return Self.X + Self.Y
    end;

    function Add(Self: Point; Other: Point): Point;
    begin
      var RX: integer := Self.X + Other.X;
      var RY: integer := Self.Y + Other.Y;
      return record X := RX; Y := RY; end
    end;

    procedure Print(Self: Point);
    begin
      WriteLn('(' + IntToStr(Self.X) + ', ' + IntToStr(Self.Y) + ')')
    end;
  end;

begin
  var A: Point := record X := 3; Y := 4; end;
  var B: Point := record X := 10; Y := 20; end;
  A.Print();
  B.Print();
  WriteLn('Sum of A: ' + IntToStr(A.Sum()));
  var C: Point := A.Add(B);
  WriteLn('A + B =');
  C.Print()
end.
```

### Unit — `Clamp` (`if` branches always get `begin` / `end`)

The language allows the compact form (see [`program-structure/units.md`](../program-structure/units.md)); **`fpas fmt` does not emit it.** Golden unit file:

```pascal
unit MyApp.Utils;

uses Std.Math;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
  begin
    return Min
  end
  else if Value > Max then
  begin
    return Max
  end
  else
  begin
    return Value
  end
end;

function IsBlank(S: string): boolean;
begin
  return Length(Trim(S)) = 0
end;
```

<details>
<summary>Before fmt (valid source — <strong>not</strong> golden output)</summary>

```pascal
unit MyApp.Utils;
uses Std.Math;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value
end;
```
</details>

---

## General

- UTF-8 output. Preserve valid Unicode in string literals and identifiers.
- Unix line endings (`\n`) in formatted output.
- Trailing newline at end of file.
- No trailing whitespace on lines.
- Case-insensitive language; emitter uses **fixed canonical spellings** (see below), not source casing.

## Line width (v2)

- **Maximum line length: 100 columns** (`MAX_LINE_WIDTH` in `crates/fpas-fmt/src/style.rs`).
- Count includes leading indentation on the line being measured.
- Lines at or below [`MAX_LINE_WIDTH`](../../../crates/fpas-fmt/src/style.rs) stay on one line; wrapping applies only when the rendered line would exceed the limit.

### Wrapping (v2, when over max width)

| Construct | Break rule |
|-----------|------------|
| `uses` clause | After commas; continuation lines indented **2 spaces** from column 0 |
| `function` / `procedure` formal lists | After `;` between parameters |
| `record` / array literals | Multi-line when over width; keep v1 semicolon rules inside |
| Long binary chains / calls | Break at lowest-precedence operator; never inside string literals |

## Indentation

- **2 spaces** per block level. No tabs.
- `begin` / `end` bodies indent one level.
- `case` arms: label on its own line; `begin` / `end` body indented one level under the label.
- `record` / `enum` type bodies indent one level.
- Continuation lines for long `uses` lists: wrap with 2-space indent from the line start (see [Line width](#line-width-v2)).

## Blocks (`begin` / `end`)

The language allows a **single statement** without `begin` / `end` after `then`, `else`, `do`, and `case` labels ([`language/control-flow/README.md`](../language/control-flow/README.md)). The formatter **always** emits an explicit `begin` / `end` wrapper anyway. We are not changing the language — only canonical output.

| Construct | Formatter output |
|-----------|------------------|
| `if` / `else if` / `else` branch | `begin` … `end` around the branch body |
| `for` … `do` body | `begin` … `end` |
| `while` … `do` body | `begin` … `end` |
| `case` arm body | `begin` … `end` (label, then block on following lines) |
| `case` `else` branch | `begin` … `end` |
| `function` / `procedure` body | already required — unchanged |
| program `begin` … `end.` | already required — unchanged |
| `repeat` … `until` | **no** extra wrapper — statement list stays directly under `repeat` |
| `record` / `enum` type, record literals | `record` … `end` / `enum` … `end` — not `begin` |
| nested `function` / `procedure` body | already required — unchanged |

## Blank lines

The formatter **inserts and removes** blank lines to match these rules. User-placed blank lines are not preserved.

| After | Blank lines before next section |
|-------|----------------------------------|
| `program Name;` | **exactly one** |
| `unit Qualified.Name;` | **exactly one** |
| `uses ...;` | **exactly one** |
| `type` block (after closing `end;` of the block) | **exactly one** before the next top-level section (`begin` in programs, or `function` / `procedure` / … in units) |
| last field in a `record` type (before methods) | **exactly one** before the first method |
| last statement before `end` / `end.` | none |

Inside `begin` … `end` blocks: **no** extra blank lines between consecutive statements unless we add a separate rule later.

---

## Keywords and builtins

Emit lowercase keywords: `program`, `unit`, `uses`, `begin`, `end`, `function`, `procedure`, `var`, `mutable`, `const`, `type`, `if`, `then`, `else`, `case`, `of`, `for`, `to`, `downto`, `in`, `do`, `while`, `repeat`, `until`, `return`, `panic`, `break`, `continue`, `and`, `or`, `not`, `xor`, `div`, `mod`, `shl`, `shr`, `public`, `private`, `record`, `enum`, `array`, `dict`, `result`, `option`, `ok`, `error`, `some`, `none`, `try`, `go`, `with`, `true`, `false`.

Boolean and enum variant constructors in expressions: `Ok`, `Error`, `Some`, `None` (Pascal-style mixed case for std-like variants).

## Identifiers

- Preserve **user identifier spelling** from the AST (`Token::Ident` path): `MyApp`, `writeLn` stay as parsed.
- Qualified names: `Std.Console`, `MyLib.Utils.Helper` — dot-separated, no extra spaces.

## Literals

| Kind | Rule |
|------|------|
| `integer` | Decimal only. No `$` hex, no `_` separators. |
| `real` | Shortest decimal that re-parses to the same `f64` (implementation picks one stable rule, e.g. no unnecessary trailing zeros). |
| `string` | Single-quoted Pascal strings. Escape `'` as `''`. Prefer `#` char codes only when required for unprintable content. |
| `string` | As the parser represents it (explicit `string` typing in source). |

## Semicolons

Semicolons are **separators**, not terminators:

- Between statements in a block: `;` after each statement except the last before `end`.
- No semicolon immediately before `end`, `else`, or `until`.
- Declarations in `type` blocks and unit/program headers: `;` between siblings; no trailing `;` before closing `end` of a nested block.
- `case` arm labels: `;` after each arm’s closing `end` (including the last arm before `else`); `else` branch follows [`language/control-flow/case-of-intro.md`](../language/control-flow/case-of-intro.md).
- Fields inside a `record` type: `;` after **every** field, including the last field before `end`, a blank line, or methods (matches existing FPAS sources).

## Spacing

- One space after keywords that introduce a clause: `if cond then`, `for i := 1 to 10 do`, `while cond do`.
- No space before `:` in type annotations (`name: integer`).
- One space around binary operators except `.` (field access) and `..` (ranges).
- Unary `not` / unary `-`: **one space** before the operand (`not x`, `-1`).
- Empty parameter lists: `()` not omitted.
- `uses` clause: comma-separated, one space after comma.

---

## More examples — `record` types (snippet)

**Also golden output** — shape of a `type` section inside a formatted file. See **PointExample** above for a full program.

### One field

```pascal
type
  IdBox = record
    Value: integer;
  end;
```

### Five fields

```pascal
type
  Person = record
    Id: integer;
    Name: string;
    Age: integer;
    Active: boolean;
    Score: real;
  end;
```

### Fields with defaults

```pascal
type
  Config = record
    Host: string := 'localhost';
    Port: integer := 8080;
    Retries: integer := 3;
  end;
```

### Record literal (expression)

Single line when it fits; multi-field literals use `;` between fields, no trailing `;` before `end`:

```pascal
record X := 3; Y := 4; end
```

```pascal
record Host := 'api'; Port := 443; Retries := 5; end
```

### Long `uses` (wrapped, v2 golden)

When the `uses` line exceeds 100 columns, break after commas:

```pascal
program LongUses;

uses
  Std.Console, Std.Conv, Std.Array, Std.Dict, Std.Option, Std.Result, Std.String,
  MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two;

begin
  WriteLn('ok')
end.
```

### Wrapped record literal (v2 golden)

```pascal
record
  Host := 'api.example.com';
  Port := 443;
  Retries := 5;
  TimeoutSeconds := 30;
end
```

---

## More examples — other types (snippet)

```pascal
type
  Color = enum
    Red;
    Green;
    Blue;
  end;

  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;

  IntBox = Box of integer;
```

---

## Types (summary)

- `array of T`, `dict of K to V`, `Result of T, E`, `Option of T`.
- Generics: `Box<T>`, usage `Box of string`, multiple params `Pair of integer, string`.
- Enum variants with data: `Circle(Radius: real);`

## Expressions (summary)

- Parentheses: omit redundant parens where parser precedence is unambiguous; always emit parens present in `Expr::Paren`.
- Function/procedure calls: `Name(arg1, arg2)` — commas in calls, semicolons only in declarations.

## Comments

**All comments are preserved** when formatting with source text ([`format_source`](../../../crates/fpas-fmt/src/lib.rs) / `fpas fmt`). That includes `///` doc lines, `//` line comments, and `{ }` / `(* *)` block comments — whether they appear before declarations, before `uses` / `begin`, between statements, or at end of line after code.

The formatter may **normalize** comment text (for example `CRLF` → `LF`, trim trailing spaces on a comment line) but must not delete any comment.

Placement after formatting follows emission anchors: leading comments stay on their own lines before the nearest following construct; end-of-line comments stay on the same line after the statement or declaration they trailed in source. One blank line after a leading doc/block group before a top-level declaration or unit/program header (see [Blank lines](#blank-lines)).

[`format_compilation_unit`](../../../crates/fpas-fmt/src/lib.rs) without source cannot recover comments from the AST alone — use [`format_source`](../../../crates/fpas-fmt/src/lib.rs) when comments must be kept.

**Golden tests:** [`comments_unit.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_unit.expected.fpas), [`comments_program.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_program.expected.fpas), [`comments_block_styles.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_block_styles.expected.fpas) (see [`golden_output.rs`](../../../crates/fpas-fmt/tests/golden_output.rs)).

## Intentional diffs from source

The formatter **normalizes** valid input. These changes are deliberate (not bugs):

| Source may have | Formatted output |
|-----------------|------------------|
| Any comment (`///`, `//`, `{ }`, `(* *)`) with [`format_source`](../../../crates/fpas-fmt/src/lib.rs) | Preserved (text may be normalized; placement follows anchor rules in [Comments](#comments)) |
| Keyword casing (`PROGRAM`, `Begin`, `WRITELN`) | Lowercase keywords; identifiers keep source spelling |
| Hex integers (`$FF`) or digit separators (`1_000`) | Decimal literals only |
| Optional single-statement branches (`if x then return y`) | Always `begin` … `end` around branch bodies |
| User-placed blank lines | Only the fixed rules in [Blank lines](#blank-lines) |
| `uses` on same line as header | Header blank line + `uses` on its own line |
| Extra parentheses from parse tree | May differ where precedence makes them redundant |
| `uses` unit name casing (`Std.array`) | Canonical qualified id spelling from the AST |

## Non-goals (v1 and later)

- Configurable style (`.fpasfmt.toml`, line width, indent size, keyword case) — **one official style only**; no per-project overrides.
- Automatic formatting (`--watch`, LSP format-on-save) — user runs `fpas fmt` explicitly.
- Preserving blank lines between user-chosen sections (except the fixed rules above).
- Sorting `uses` clauses or declaration order.
- Formatting invalid or partial syntax (recovery).

## See also

- [Tools index](README.md)
- [CLI reference](../program-structure/cli.md)
