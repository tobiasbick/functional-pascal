# Std.Text.Regex

`Std.Text.Regex` compiles Unicode regular expressions and searches, captures,
replaces and splits FPAS strings.

Import with `uses Std.Text.Regex as Regex;`. Compile a pattern once and pass it
to the operations below. There are no parallel string-pattern shortcuts.

## Quick reference

| Kind | Signature | Description |
| --- | --- | --- |
| function | `DefaultOptions(): Options` | Default case-sensitive Unicode search options |
| function | `Compile(Expression: string; Options: Options): result of (Pattern, RegexError)` | Compile an immutable managed pattern |
| function | `IsMatch(Pattern: Pattern; Text: string): result of (boolean, RegexError)` | Test whether a match exists anywhere |
| function | `Find(Pattern: Pattern; Text: string; Start: integer): result of (option of Match, RegexError)` | First match at or after a Unicode scalar position |
| function | `FindAll(Pattern: Pattern; Text: string; Start: integer): result of (array of Match, RegexError)` | All nonoverlapping matches, including captures |
| function | `Replace(Pattern: Pattern; Text: string; Replacement: string; Limit: integer): result of (string, RegexError)` | Expand capture references; zero limit replaces all |
| function | `Split(Pattern: Pattern; Text: string; Limit: integer): result of (array of string, RegexError)` | Split into at most Limit parts; zero returns all |
| function | `Escape(Text: string): result of (string, RegexError)` | Quote literal text for use inside a pattern |
| function | `GroupNames(Pattern: Pattern): result of (array of option of string, RegexError)` | Capture names in numbered-group order, including group zero |

`Pattern` is an opaque immutable value. Assignment, capture and task transfer
share its compiled storage. Its last owner releases that storage automatically;
there is no handle registry, explicit close or process-global pattern cache.
Equality compares compiled identity: aliases are equal, independent compilations
are not. Patterns have no ordering. Printing shows the source expression.

## Options and pattern syntax

`Options` has six public boolean fields. `DefaultOptions()` sets all to false:

| Field | Meaning |
| --- | --- |
| `CaseInsensitive` | Unicode-aware case-insensitive matching |
| `MultiLine` | `^` and `$` also match line boundaries |
| `DotMatchesNewLine` | `.` also matches newline |
| `IgnoreWhitespace` | Ignore pattern whitespace and `#` comments outside escaped text |
| `SwapGreed` | Reverse the default meaning of greedy and lazy quantifiers |
| `CrLf` | Treat CRLF as a line ending for multiline anchors |

The engine is Rust's [`regex`](https://docs.rs/regex/latest/regex/#syntax).
It supports Unicode classes (`\p{L}`, `\p{Greek}`), character classes,
alternation, repetitions, lazy quantifiers, anchors, word boundaries, numbered
groups, named groups (`(?P<name>...)` or `(?<name>...)`), noncapturing groups and
scoped or inline flags such as `(?i)` and `(?m:...)`. FPAS string literals do not
escape backslashes: write `'\d+'` directly. Unicode matching is enabled by
default; inline flags may select ASCII classes where valid UTF-8 is preserved.

Lookaround and pattern backreferences are not part of this dialect; compilation
returns `InvalidPattern` for them. There is one engine and one pattern dialect.
Individual searches avoid exponential backtracking. Repeated searches can still
require substantial work; text, compiled size and result limits are enforced.

`IsMatch` tests whether a match exists anywhere. Use `\A` and `\z` in the
pattern when the complete string must match.

## Positions, matches and groups

All public positions count **Unicode scalar values**, starting at zero, just
like FPAS string indexing and `Length()`. They do not count UTF-8 bytes or
grapheme clusters. `Start` may equal `Text.Length()` to search its final boundary.
Negative positions and positions beyond that boundary return `InvalidStart`.
Searching from a position retains the complete text as anchor and word-boundary
context: searching `^a` in `'ba'` from 1 does not make that `a` a string start.

| Record | Public fields |
| --- | --- |
| `Span` | `Start: integer`, `Length: integer`, `Text: string` |
| `Capture` | `Name: option of string`, `Value: option of Span` |
| `Match` | `Span: Span`, `Groups: array of Capture` |

`Match.Span` describes the whole match. `Groups[0]` contains the whole match
again; subsequent indexes are the pattern's numbered captures. Every declared
group has a slot, even when it does not participate. Its `Value` is then `None`;
a participating empty capture has `Some` with length zero. `Name` is `None`
for unnamed groups. `GroupNames` returns these same names and indexes without
searching any text.

`Find` returns the first leftmost match or `None`. `FindAll` returns matches
in order without overlap. Empty matches advance to the next Unicode boundary
and include the final boundary when appropriate. An empty match immediately
after a nonempty match ending at the same position is suppressed, following
the engine's iterator semantics. An empty pattern on `'é🙂'` therefore yields
positions 0, 1 and 2.

## Replacement and splitting

`Replace` expands `$0`, `$1`, `$name` and `${name}` in replacement text. `$$`
inserts a literal dollar sign. Unbraced names take the longest ASCII sequence
of letters, digits and underscores; `${1}suffix` disambiguates a suffix.
Unknown or nonparticipating groups expand to empty text. A dollar sign that
does not start a reference stays literal. Replacement text is not a pattern;
`Escape` is for quoting literal text inside a pattern, not a replacement.

`Replace`'s `Limit` is the maximum replacements. Zero replaces all matches;
a positive number stops after that many. `Split`'s `Limit` is the maximum
parts; zero returns all parts, and one returns the original string as one part.
Separators are omitted, captures do not become extra parts, and empty parts
are preserved. Negative limits return `InvalidLimit`.

## Errors and resource limits

| `RegexError` variant | Payload |
| --- | --- |
| `InvalidPattern` | `Detail: string` |
| `InvalidStart` | `Start: integer; Length: integer` |
| `InvalidLimit` | `Limit: integer` |
| `LimitExceeded` | `Name: string; Limit: integer` |

Patterns are limited to 64 KiB of UTF-8 source, 8 MiB of compiled engine size,
256 capture slots including group zero, and nesting depth 250. The engine's
lazy DFA cache limit is 2 MiB per search cache. Text and replacement arguments are each
limited to 16 MiB. Returned text totals at most 16 MiB per operation, counting
copied match and capture texts separately. Results are limited to 100,000 items,
counting matches and capture slots together, split parts, or replacements.
Exceeding a result limit returns an error instead of a truncated success.
Pattern nesting and syntax failures use `InvalidPattern`; the other named
bounds use `LimitExceeded`. These limits are independent of OS memory capacity.

## Example

```pascal id=regex-captures run
program RegexExample;
uses Std.Text.Regex as Regex, Std.Test as Test;
begin
  var Options := Regex.DefaultOptions();
  Options.CaseInsensitive := true;
  const Pattern := Regex.Compile('(?P<name>\p{L}+)=([0-9]+)', Options).Unwrap();
  const Found := Regex.Find(Pattern, '🙂 Größe=42', 0).Unwrap().Unwrap();
  Test.AssertEquals(2, Found.Span.Start);
  Test.AssertEquals('Größe', Found.Groups[1].Value.Unwrap().Text);
  Test.AssertEquals('42', Found.Groups[2].Value.Unwrap().Text);
  Test.AssertEquals('Größe:42', Regex.Replace(Pattern, 'Größe=42', '${name}:$2', 0).Unwrap());
end.
```

## Implementation (contributors)

Declarations are registered in
`crates/fpas-sema/src/std_registry/loaded/regex.rs`; generated editor declarations
are in `lib/api/Std/Text.Regex.fpas`. Intrinsics are implemented by
`crates/fpas-runtime/src/stdlib/regex/` and `regex.rs`.
Compiled patterns are immutable reference-counted runtime values and support
debugger inspection and detached calls without external effects.

## See also

- [Text and parsing](README.md)
- [String positions and search](../../language/types/string/search.md)
- [Error handling](../../language/error-handling/README.md)
