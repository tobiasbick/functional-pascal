# Source formatting

`fpas fmt` formats complete FPAS sources using one fixed style. Formatting uses
the existing parser without binding, resolving references, compiling or executing
the FPAS program. Invalid syntax produces diagnostics and no rewritten sources.
There are no style configuration files or alternate styles.

## Command

```sh
fpas fmt examples/hello/main.fpas
fpas fmt --check --list examples
fpas fmt --stdout examples/hello/main.fpas
fpas fmt --check examples/hello/Hello.fpasproj --report json
fpas fmt --check Example.slnx --configuration Release
```

Supply one or more explicit `.fpas`, directory, `.fpasproj` or `.slnx` paths.
Paths resolve against the invocation directory; quote paths containing spaces.
Use `--` before a path starting with `-`. Response-file expansion, stdin, globs
and implicit current-directory discovery are not supported.

- Default: rewrite changed files. Unchanged files retain their modification time.
- `--check`: write no files; return 2 if any source would change.
- `--check --list`: print only changed absolute paths, one per line, to stdout.
  Without `--list`, check explains pending changes on stderr.
- `--stdout`: print exactly one selected source; write no files. Cannot combine
  with `--check` or `--report`.
- `--report json`: produce one JSON document, including argument and input
  failures. Cannot combine with `--list` or `--stdout`.

Directories recurse, skipping `.git`, `.temp-data`, `bin`, `obj`, `node_modules`
and symbolic links. Explicit paths and their parent directories must be regular,
non-link paths. Inputs are deduplicated and sorted by absolute path.

Projects select their evaluated MSBuild `Compile` items, including linked files
and configuration conditions. Solutions select enabled FPAS projects using their
configuration/platform mappings. C#/F# sources are not formatted. Evaluation
uses the installed project SDK; it does not invoke build, restore or source
generation targets. Ordinary MSBuild evaluation/SDK resolution still occurs.
Formatting a directory selects its regular sources independently of project
exclusions; choose a project when its `Compile` selection is required.

The complete operation has a 120-second default deadline. `--timeout` accepts
1..3600 seconds. The combined source input limit is 64 MiB. Cancellation stops
owned evaluation processes. All inputs are parsed and revalidated before any
write; a syntax failure leaves every source untouched. Each changed file is
staged beside its destination and replaced atomically after an exact-byte stale
check. Unix file permissions are retained. An I/O failure or cancellation during
publication may leave earlier files formatted; JSON `written` fields identify
them. This is not a multi-file filesystem transaction.

Exit codes: 0 success/help, 1 source/evaluation/I/O/deadline failure, 2 formatting
changes under `--check` or invalid arguments, 130 cancellation. JSON distinguishes
these using `success`, `mode`, `files`, `diagnostics` and an optional `error`.
Each file entry has `path`, `changed` and `written`; paths are absolute. Check
drift has `success: false`, changed entries and no source diagnostic. Diagnostic
locations refer to the original source. JSON and `--list` own stdout; human
diagnostics go to stderr.

## Fixed rules

- UTF-8 without BOM, LF line endings, one final newline. Trim trailing
  whitespace outside string literals.
- Two spaces per structural level; no indentation tabs.
- A 100-display-column target. Wide characters count as two cells; combining
  sequences do not add cells. Indivisible names, literals and comments can exceed
  the target. Tokens are never split.
- Preserve keyword and identifier spelling, including escaped identifiers.
  FPAS is case-sensitive; formatting does not repair keyword casing.
- Preserve literal spelling, including numeric bases, separators, suffixes,
  quote escaping and character codes. Normalize line endings in multiline
  strings to LF and preserve their spaces and tabs.
- Preserve all tokens, explicit parentheses, named endings, semicolons,
  declaration/import order and scope boundaries. Do not add or remove `begin`.
- Put routine bodies, control bodies, record/union/enum/interface members and
  case arms on structural lines. Bodies indent one level. A case statement's
  arm body indents another level below its `when`/`else` header.
- Record updates always expand after `with`, including single-field updates.
  Each field assignment occupies its own line, indented one level; `end with`
  returns to the update's owning indentation. Nested updates add another level.
- Named routine `begin` aligns with its declaration. Anonymous routines keep
  `begin` after their signature when no contracts intervene; empty bodies can
  remain compact. Contracts and generic constraints occupy separate lines.
- Separate the compilation-unit header, imports and following section with one
  blank line. Separate top-level declarations, except adjacent bindings of the
  same kind/visibility. Separate record fields from methods and methods from
  one another. A unit's final ending follows its last declaration directly.
- Separate a completed block/control statement from the next sibling with one
  blank line unless the next sibling is `const`/`var`. No blank is inserted before
  structural continuations or closers.
- Preserve authored sections between sibling statements and adjacent bindings,
  including before `const` and `var`; normalize repeated blank lines to one.
  Comments alone do not create a section. A blank after a comment still keeps
  that comment detached from the following declaration.
- One space around binary operators and after commas/semicolons within lists.
  No spaces around member dots, ranges, generic angle delimiters or nullable
  suffixes. Type annotations use `Name: Type`; prefix signs attach to operands,
  and `not`/`await` use a space. Calls and indexes attach to their receivers.

## Wrapping and comments

Use one blank line between groups that perform different jobs, such as input
collection, validation, processing and reporting. Keep related bindings and
operations together. The formatter preserves these authored sections rather
than inferring their meaning.

Import and formal parameter lists retain authored line groups. Expanded lists
start their contents on a new line indented two spaces from the owning line.
Closing delimiters return to the owning indentation. Calls, indexes and array
literals also expand automatically when over the width target. Nested lists are
measured at their structural indentation. A multiline element expands its parent;
an overlong authored argument group gains additional breaks. No trailing separator
is introduced.

Binary expressions retain authored operand breaks and gain breaks for the width
target. Logical `and`, `or` and `xor` begin continuation lines; other operators
end the preceding line. A multiline statement condition puts `then` on its own
line aligned with `if`/`elsif`. Conditional expressions expand their branches
when multiline or over the target. Case expressions use structural arm lines.

All `//` comments retain their order and text, except trailing whitespace.
Standalone comments stay before the following token; trailing comments stay on
the preceding code line. A comment after `then` or `do` can stay on that clause's
line. Contiguous declaration documentation stays attached; an authored blank
after a comment keeps it detached. Before returning formatted output, the
formatter reparses it and verifies tokens, comments and documentation attachment.

## Canonical example

This complete source is already formatted:

```pascal
program FormatExample;

uses System.Console as Console;

function Clamp(Value: int; Minimum: int; Maximum: int): int;
begin
  if Value < Minimum then
    return Minimum;
  elsif Value > Maximum then
    return Maximum;
  else
    return Value;
  end if;
end function;

begin
  Console.WriteLine(Clamp(7, 0, 10));
end;
end program;
```

## See also

- [Tools](README.md)
- [CLI reference](../program-structure/cli.md)
- [Comments and documentation](../language/basics/comments.md)
