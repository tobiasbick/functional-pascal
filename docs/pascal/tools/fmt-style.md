# Formatter style rules

Canonical output rules for the AST pretty-printer. These are **normative for `fpas fmt`** once implemented. The emitter encodes them; this file is the human-readable spec.

**Status:** **complete** (2026-06). Normative for [`fpas fmt`](../../../crates/fpas-cli/src/cli_fmt/) and the [editor formatter](editor-integration.md#formatting). Edit golden examples when the style changes; both entry points must match them.

**How to read this file**

| Section | What the code blocks show |
|---------|---------------------------|
| [Formatted output](#formatted-output-fpas-fmt) | **After `fpas fmt`** — complete `.fpas` files (golden output). This is what the formatter must produce. |
| [More examples — snippets](#more-examples--record-types-snippet) | **After `fpas fmt`** — same rules, but only a `type` slice (not a full file). |
| Rules below (indent, semicolons, …) | Textual spec; if a rule disagrees with a golden example, **fix the example or the rule**, then implement. |

There is **no** “messy input” column in this doc yet. Source before formatting may omit `begin` / `end`, use `WRITELN`, or extra blank lines — those are normalized in the golden blocks. Comments are preserved (see [Comments](#comments)).

Spec links: [`language/basics/README.md`](../language/basics/README.md), [`language/control-flow/README.md`](../language/control-flow/README.md), [`.agents/skills/fpas-authoring/SKILL.md`](../../../.agents/skills/fpas-authoring/SKILL.md).

---

## Formatted output (`fpas fmt`)

**Golden output.** Each block below is a **complete file** as written to disk after `fpas fmt` — including every `begin` / `end` the formatter inserts.

Visual checklist:

- `program` / `unit` header → **one blank line** → `uses` (if any) → **one blank line** → rest
- Every **program** ends with `begin` … `end.` (period on `end`)
- Every **function** / **procedure** / **method** body: `begin` … `end function;` or `end procedure;`
- Anonymous routine expressions use `end function` / `end procedure`, and record
  updates use `end with`; their enclosing syntax owns any final terminator
- Every **unit** ends with `end unit;` after its declarations
- Every **`if` / `elsif` / `else`** and **`for` / `while`** body is a statement list,
  indented one level and closed by `end if;`, `end for;`, or `end while;`
- **`case` arms** start with `when`, hold statement lists indented one level
  under the arm header, and share `end case;` with the optional `else` arm
- **`repeat` … `until`**: no extra `begin` / `end` around the body
- Inside `begin` … `end`, one blank line separates completed `end;` blocks from the next
  non-variable statement

### Program — minimal

```pascal
program Hello;

begin
  WriteLn('Hello, World!');
end.
```

### Program — with `uses`

```pascal
program Hello;

uses Std.Console;

begin
  WriteLn('Hello, World!');
end.
```

### Program — control flow (`if`, `case`, `for`, `while`, `repeat`)

Same file **after** fmt. Explicit scoping blocks from the source are retained:

```pascal
program ControlFlowDemo;

uses Std.Console, Std.Conv;

begin
  var X: integer := 5;
  if X > 0 then
    begin
      WriteLn('positive');
    end;
  elsif X = 0 then
    begin
      WriteLn('zero');
    end;
  else
    begin
      WriteLn('negative');
    end;
  end if;

  case X of
    when 1:
      begin
        WriteLn('one');
      end;
    when 2, 3:
      begin
        WriteLn('two or three');
      end;
    when 10..20:
      begin
        WriteLn('ten to twenty');
      end;
    else
      begin
        WriteLn('other');
      end;
  end case;

  for I: integer := 1 to 3 do
    begin
      WriteLn(IntToStr(I));
    end;
  end for;

  while X < 10 do
    begin
      X := X + 1;
    end;
  end while;
  mutable var N: integer := 0;
  repeat
    WriteLn(IntToStr(N));
    N := N + 1;
  until N >= 3;
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
      return Self.X + Self.Y;
    end function;

    function Add(Self: Point; Other: Point): Point;
    begin
      var RX: integer := Self.X + Other.X;
      var RY: integer := Self.Y + Other.Y;
      return record
        X := RX;
        Y := RY;
      end;
    end function;

    procedure Print(Self: Point);
    begin
      WriteLn('(' + IntToStr(Self.X) + ', ' + IntToStr(Self.Y) + ')');
    end procedure;
  end record;

begin
  var A: Point := record
    X := 3;
    Y := 4;
  end;
  var B: Point := record
    X := 10;
    Y := 20;
  end;

  A.Print();
  B.Print();
  WriteLn('Sum of A: ' + IntToStr(A.Sum()));
  var C: Point := A.Add(B);
  WriteLn('A + B =');
  C.Print();
end.
```

### Unit — `Clamp` (named conditional ending)

Branch statements are indented directly under their clause (see
[`program-structure/units.md`](../program-structure/units.md)). Golden unit file:

```pascal
unit MyApp.Utils;

uses Std.Math;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min;
  elsif Value > Max then
    return Max;
  else
    return Value;
  end if;
end function;

function IsBlank(S: string): boolean;
begin
  return Length(Trim(S)) = 0;
end function;
end unit;
```

<details>
<summary>Before fmt (valid source — <strong>not</strong> golden output)</summary>

```pascal
unit MyApp.Utils;

uses Std.Math;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min;
  elsif Value > Max then
    return Max;
  else
    return Value;
  end if;
end function;
end unit;
```
</details>

---

## General

- UTF-8 output. Preserve valid Unicode in string literals and comments. Identifiers use ASCII letters, digits, and `_` only.
- Unix line endings (`\n`) in formatted output.
- Trailing newline at end of file.
- No trailing whitespace on lines.
- Case-insensitive language; emitter uses **fixed canonical spellings** (see below), not source casing.

## Line width (v2)

- **Maximum line length: 100 columns** (`MAX_LINE_WIDTH` in `crates/fpas-fmt/src/style.rs`).
- Count includes leading indentation and uses terminal-style Unicode display columns: combining
  marks have width zero and wide characters have width two.
- Lines at or below [`MAX_LINE_WIDTH`](../../../crates/fpas-fmt/src/style.rs) stay on one line; wrapping applies only when the rendered line would exceed the limit.

### Wrapping (v2, when over max width)

| Construct | Break rule |
|-----------|------------|
| `uses` clause | After commas; continuation lines indented **2 spaces** from column 0 |
| `function` / `procedure` formal lists | After `;` between parameters |
| Record literals with fields | Always multi-line; keep semicolons after every field |
| Array literals | Multi-line when over width |
| Long binary chains / calls | Break at lowest-precedence operator; never inside string literals |
| Postfix chains (`.Field` / `[Index]` / `.Method(...)`) | Break before each suffix; indent continuations **2 spaces** from the expression base column |

## Indentation

- **2 spaces** per block level. No tabs.
- `begin` / `end` bodies indent one level.
- `case` arms: `when Labels [if Guard]:` on its own line; body statements
  indented one level under the header. `else` aligns with `when`; `end case;`
  aligns with `case`.
- `record` / `enum` type bodies indent one level.
- Continuation lines for long `uses` lists: wrap with 2-space indent from the line start (see [Line width](#line-width-v2)).

## Blocks (`begin` / `end`)

An `if`, `case`, `for`, or `while` contains scoped statement lists. The formatter
indents each body one level without adding `begin` / `end`. Explicit compound
statements remain as nested scopes. `elsif` continues the same conditional;
an `if` inside an `else` list keeps its separate ending. Empty control bodies
are rejected by the parser and must be written as `null;`.

| Construct | Formatter output |
|-----------|------------------|
| `if` / `elsif` / `else` | indented branch lists and one `end if;` |
| `for` … `do` | indented statement list and `end for;` |
| `while` … `do` | indented statement list and `end while;` |
| explicit compound statement | `begin` … `end;`, preserving its nested scope |
| `case` arm body | `when` header, then an indented statement list |
| `case` `else` branch | `else`, then an indented statement list |
| `case` statement | arm lists followed by `end case;` |
| named `function` / `procedure` body | `begin` … `end function;` / `end procedure;` |
| anonymous `function` / `procedure` expression | `begin` … `end function` / `end procedure`, without its own terminator |
| record update expression | `Base with Field := Value; end with`, without its own terminator |
| program body | `begin` … `end.` |
| `repeat` … `until` | statement list directly under `repeat` |
| `record` / `enum` type | `record` … `end record;` / `enum` … `end enum;` |
| record literal | `record` … `end` |
| unit | declarations followed by `end unit;` |

## Blank lines

The formatter **inserts and removes** blank lines to match these rules. User-placed blank lines are not preserved.

| After | Blank lines before next section |
|-------|----------------------------------|
| `program Name;` | **exactly one** |
| `unit Qualified.Name;` | **exactly one** |
| `uses ...;` | **exactly one** |
| `type` block (after the final declaration terminator) | **exactly one** before the next top-level section (`begin` in programs, or `function` / `procedure` / … in units) |
| last field in a `record` type (before methods) | **exactly one** before the first method |
| sibling statement ending in `end;` or a named control ending | **exactly one**, unless the next sibling is `var` or `mutable var` |
| last statement before `end` / `end.` | none |

This statement-spacing rule applies only between sibling statements. It never inserts a blank line
before structural continuations or closers such as `elsif`, `when`, `else`, `until`, `end`, or `end.`, and it does not
separate `case` arms. Leading comments stay attached to the following statement after the blank line.

---

## Keywords and builtins

Emit lowercase keywords: `program`, `unit`, `uses`, `begin`, `end`, `function`, `procedure`, `var`, `mutable`, `const`, `type`, `if`, `then`, `elsif`, `else`, `null`, `case`, `when`, `of`, `for`, `to`, `downto`, `in`, `do`, `while`, `repeat`, `until`, `return`, `panic`, `break`, `continue`, `and`, `or`, `not`, `xor`, `div`, `mod`, `public`, `record`, `enum`, `array`, `channel`, `dict`, `result`, `option`, `ok`, `error`, `some`, `none`, `try`, `go`, `with`, `static`, `property`, `event`, `read`, `write`, `comparable`, `numeric`, `printable`, `self`, `nil`, `true`, `false`.

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

Semicolons are **terminators**:

- Every statement and declaration ends with `;`, including the last statement
  before `end`, `else`, or `until`. The program's final `end.` keeps its period.
- Named declarations have one matching ending: `end function;`, `end procedure;`,
  `end record;`, `end enum;`, or `end unit;`. The ending contains the declaration
  terminator; no additional `;` follows it.
- Control statements end with `end if;`, `end case;`, `end for;`, or `end while;`. Each body
  statement has its own `;`, including the last before the named ending.
- Each conditional branch is terminated before `elsif` or `else`. A scoped
  `begin` ... `end;` closes only that compound statement.
- A `repeat` body's final statement ends with `;`, and `until Condition;`
  terminates the loop.
- Expressions have no terminator of their own. An anonymous routine body uses
  terminated statements and closes with `end function` or `end procedure`;
  record updates close with `end with`. In arguments, the named ending is
  followed by `,` or `)` without `;`. An enclosing statement or declaration
  supplies its own terminator. Record-update fields retain every `;`.
- Formal parameter lists keep `;` between parameters and have no trailing
  separator before `)`.
- `case` arms: every body statement ends with `;`, including the last one
  before `when`, `else`, or `end case`. Arm headers have no terminating `;`.
  The `else` arm follows [`language/control-flow/case-of-intro.md`](../language/control-flow/case-of-intro.md).
- Fields inside a `record` type: `;` after **every** field, including the last field before `end record`, a blank line, or methods (matches existing FPAS sources).
- Preserve `public` on exported unit declarations and individual record
  members. Private declarations and members have no modifier. A routine
  modifier appears before `static`.

## Spacing

Record updates keep compact field assignments when there are no attached
comments. With comments, fields are indented one level below `with`, and
`end with` returns to the expression's enclosing indentation. Comments before,
inside, and after a named expression ending are preserved. An empty anonymous
routine stays compact unless comments require separate lines.

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
  end record;
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
  end record;
```

### Fields with defaults

```pascal
type
  Config = record
    Host: string := 'localhost';
    Port: integer := 8080;
    Retries: integer := 3;
  end record;
```

### Record literal (expression)

Non-empty record literals are always multi-line. Every field keeps its trailing `;`, including the
last field before `end`:

Empty record literals stay on one line with exactly one space:

```pascal
record end
```

```pascal
record
  X := 3;
  Y := 4;
end
```

```pascal
record
  Host := 'api';
  Port := 443;
  Retries := 5;
end
```

### Long `uses` (wrapped, v2 golden)

When the `uses` line exceeds 100 columns, break after commas:

```pascal
program LongUses;

uses
  Std.Console, Std.Conv, Std.Arrays, Std.Dictionaries, Std.Options, Std.Results, Std.String,
  MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two;

begin
  WriteLn('ok');
end.
```

### Record literal (v2 golden)

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
  end enum;

  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end enum;

  IntBox = Box of integer;
```

---

## Types (summary)

- `array of T`, `channel of T`, `task of T`, `dict of K to V`, `Result of T, E`, `Option of T`.
- Generics: `Box<T>`, usage `Box of string`, multiple params `Pair of integer, string`.
- Enum variants with data: `Circle(Radius: real);`

## Expressions (summary)

- Parentheses: omit redundant parens where parser precedence is unambiguous; always emit parens present in `Expr::Paren`.
- Logical expressions follow the [operator table](../language/basics/operators.md#operator-precedence).
  Mixed `and`, `or`, and `xor` operands require parentheses. Comparisons bind
  more tightly than `not`; negated comparison operands and nested comparisons
  retain their required grouping. These rules also apply when a long expression wraps.
- Function/procedure calls use commas between arguments: `Name(arg1, arg2)`.
  A call used as a statement ends with `;`.

## Comments

**All `//` comments are preserved** when formatting with source text ([`format_source`](../../../crates/fpas-fmt/src/lib.rs) / `fpas fmt`), whether they appear before declarations, before `uses` / `begin`, between statements, or at end of line after code.

The formatter may **normalize** comment text (for example `CRLF` or bare `CR` → `LF`, trim trailing spaces on a comment line) but must not delete any comment.

Placement after formatting follows structural emission anchors: leading comments stay on their own
lines before the nearest following construct, including each program, routine, or closure body.
End-of-line comments stay on the same line after the compilation-unit/routine header, statement,
declaration, record/enum member, routine body, or final `end.` they trailed in source. A comment
after a control-flow clause such as `then` or `do` attaches to the following body and is emitted as
a leading body comment. A contiguous standalone comment block directly before a declaration stays
adjacent because it is Markdown documentation. When the source contains a blank line between a
comment and a declaration, formatting retains one blank line so the comment remains detached (see
[Comments and declaration documentation](../language/basics/comments.md)).

[`format_compilation_unit`](../../../crates/fpas-fmt/src/lib.rs) without source cannot recover comments from the AST alone — use [`format_source`](../../../crates/fpas-fmt/src/lib.rs) when comments must be kept. `format_source` is fallible and rejects a compilation unit that was not parsed from the exact source snapshot, including invalid or out-of-range UTF-8 spans.

**Tests:** [`comments_unit.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_unit.expected.fpas), [`comments_program.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_program.expected.fpas), [`comments_before_body.expected.fpas`](../../../crates/fpas-fmt/tests/golden/comments_before_body.expected.fpas), and the focused comment/API/layout regressions under [`crates/fpas-fmt/tests/`](../../../crates/fpas-fmt/tests/).

## Intentional diffs from source

The formatter **normalizes** valid input. These changes are deliberate (not bugs):

| Source may have | Formatted output |
|-----------------|------------------|
| Any `//` comment with [`format_source`](../../../crates/fpas-fmt/src/lib.rs) | Preserved (text may be normalized; placement follows anchor rules in [Comments](#comments)) |
| Keyword casing (`PROGRAM`, `Begin`, `WRITELN`) | Lowercase keywords; identifiers keep source spelling |
| Hex integers (`$FF`) or digit separators (`1_000`) | Decimal literals only |
| Branch and loop statement lists | Bodies indented one level, with named endings and no inserted compound wrapper |
| User-placed blank lines | Only the fixed rules in [Blank lines](#blank-lines) |
| `uses` on same line as header | Header blank line + `uses` on its own line |
| Extra parentheses from parse tree | May differ where precedence makes them redundant |
| `uses` unit name casing (`Std.Arrays`) | Canonical qualified id spelling from the AST |

## Non-goals

- Configurable style (`.fpasfmt.toml`, line width, indent size, keyword case) — **one official style only**; no per-project overrides.
- A separate formatter watch mode (`--watch`) — command-line users run
  `fpas fmt` explicitly; editors use the LSP formatter and their standard
  format-on-save setting.
- Preserving blank lines between user-chosen sections (except the fixed rules above).
- Sorting `uses` clauses or declaration order.
- Formatting invalid or partial syntax (recovery).

## See also

- [Tools index](README.md)
- [CLI reference](../program-structure/cli.md)
