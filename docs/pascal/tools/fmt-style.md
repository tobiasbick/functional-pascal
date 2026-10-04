# Formatter style rules

These rules describe the canonical output of `fpas fmt` and the editor formatter.
Both use the same AST emitter. Formatting preserves syntax structure, explicit
lexical blocks, comments, declaration order, and evaluation order. Formatting the
result again produces the same output.

Spec links: [grammar](../../specs/grammar.ebnf),
[control flow](../language/control-flow/README.md), and
[authoring guidance](../../../.agents/skills/fpas-authoring/SKILL.md).

## General

- UTF-8 output with Unix line endings and a trailing newline.
- Two spaces per indentation level; no tabs or trailing whitespace.
- Lowercase keywords; preserve user identifier spelling.
- Maximum line width is 100 display columns. Combining marks have width zero and
  wide characters have width two. String literals remain indivisible.
- Preserve parentheses represented in the AST.

## Compilation units and declarations

The header has one following blank line. Each import appears on its own line:
`uses Unit.Name as Alias;`. Imports are not sorted. A blank line separates imports
from the following declarations or main block.

Each `type`, `const`, `var`, and `mutable var` declaration repeats its keyword.
Top-level declarations start in column zero. A record or enum body is indented
one level; the closer aligns with the declaration. Exported declarations and
record members retain their `public` modifier. Private declarations have no
visibility modifier.

Programs close with `end program;`, and units with `end unit;`. Routines have a
`begin` body and close with `end function;` or `end procedure;`. Separate sibling
routines with one blank line. Nested routines use the same rules.

Separate a unit's last declaration from `end unit;` with one blank line. An
empty unit contains just its header, one blank line, and `end unit;`.

Type applications always emit parentheses after `of`, including one argument:
`array of (integer)`, `dict of (string, integer)`, `channel of (Message)`,
`task of (integer)`, `Option of (User)` and `Result of (User, string)`.
Generic declarations use the same list syntax:
`function Identity of (T)(Value: T): T;`. Constraint lists retain their declared
names, such as `of (T: Equatable)`.

## Statement bodies and semicolons

Every statement ends with `;`, including the final statement before `else`,
`elsif`, `when`, `until`, or a closer. A body with no action contains `null;`.
The formatter does not invent empty statements or omit required terminators.

Control-flow bodies are indented statement lists. The formatter does not add
`begin` wrappers. An explicit `begin ... end;` block remains explicit because it
creates a nested lexical scope.

| Construct | Closing syntax |
|-----------|----------------|
| Conditional | `end if;` |
| Case | `end case;`, with each arm introduced by `when` |
| Counted or collection loop | `end for;` |
| While loop | `end while;` |
| Repeat loop | `until Condition;` |
| Plain lexical block | `end;` |

`elsif` aligns with its owning `if`. An `else if` retains the nested conditional
and both closers. Case labels appear on their own lines; their statements are
indented one level below the label. Case `else` aligns with `case`.

A completed sibling statement whose output ends with a block closer is followed
by one blank line unless the next statement is a variable declaration. No blank
line is inserted before a structural continuation or the enclosing closer.

## Expressions and members

Expression closers have no statement semicolon of their own. Anonymous routines
close with `end function` or `end procedure` directly before an argument comma,
closing parenthesis, or the enclosing statement's terminator.

Builtin wrapper expressions emit qualified variant names: `Option.Some(Value)`,
`Option.None`, `Result.Ok(Value)` and `Result.Error(Message)`. Builtin wrapper
type and variant names use these canonical spellings.

Named record constructors use comma-separated field assignments:
`Point(X := 3, Y := 4)`. Supplied fields keep their written evaluation order;
long constructor argument lists follow the ordinary call-wrapping rules.

Obsolete anonymous record syntax is rejected before formatting. Record updates
retain semicolon-separated assignments and close with `end with`.

Record fields and enum members each keep their trailing semicolon. Insert one
blank line between the last field and the first record method. Routine modifiers
appear before `static`.

## Spacing and wrapping

- No space before a type annotation colon; one space after it.
- One space around binary operators, except field dots and range `..`.
- Unary minus is adjacent to its operand; `not` has a following space.
- Formal parameters are separated by `;`; actual arguments by commas.
- Empty parameter and argument lists emit `()`.
- Long formal lists break after parameter separators.
- Arrays wrap when they exceed the width limit, without a blank line before `]`.
- Long calls and binary chains break at expression boundaries.
- Long postfix chains break before suffixes, with two additional spaces of
  continuation indentation.

## Literals

Integers emit decimal digits without separators or hexadecimal notation. Reals
use a stable decimal representation that reparses to the same value. Strings use
Pascal single quotes, doubled embedded quotes, and character codes for control
characters that require them.

## Comments

`format_source` and `fpas fmt` preserve all `//` comments. Comment line endings
and trailing whitespace may be normalized. Leading comments stay with the next
construct; end-of-line comments stay on the code line they trailed. Comments
before routine or closure bodies remain with that body's `begin`.

A standalone comment block adjacent to a declaration remains attached as Markdown
documentation. A source blank line separating that block from the declaration
remains one blank line. AST-only formatting cannot recover comments without the
matching source snapshot; invalid source spans are rejected.

## Formatted output (`fpas fmt`)

The following complete files are canonical output. Additional golden fixtures
live in [`crates/fpas-fmt/tests/golden/`](../../../crates/fpas-fmt/tests/golden/).

### Program with an import

```pascal
program Hello;

uses Std.Console as Console;

begin
  Console.WriteLn('Hello, World!');
end program;
```

### Branch lists and an explicit block

```pascal
program Branches;

begin
  if true then
    null;
  elsif false then
    null;
  else
    begin
      var Value: integer := 1;
    end;
  end if;
end program;
```

### Record declaration and construction

```pascal
program T;

type Point = record
  X: integer;
  Y: integer;
end record;

begin
  var A: Point := Point(X := 3, Y := 4);
end program;
```

## Non-goals

The formatter has one official style: no per-project settings, import sorting,
or declaration reordering. Invalid or partial syntax is not formatted.

## See also

- [Tools index](README.md)
- [Editor integration](editor-integration.md)
- [CLI reference](../program-structure/cli.md)

## Operators

Callable invocation suffixes follow the same compact/wrapped chain rules as
fields, indices and member calls: `MakeAdder(3)(5)` and `Callbacks[0](42)`.
Parentheses around callable targets are preserved. Explicit result consumption
uses `discard Expression;`, with one space after the keyword. Comments in callable
arguments and anonymous routine bodies remain attached through formatting.

Formatting follows the [operator table](../language/basics/operators.md).
Comparisons bind above `not`, which binds above `and`, `or`, and `xor`.
Keep parentheses around mixed logical operators and non-left-associated
arithmetic. Comments between operands retain their order. The formatter rejects
ambiguous mixed logical chains and obsolete infix shifts along with other parse
errors; it does not choose a migration meaning.
