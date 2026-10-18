# API lookup from the CLI

Plan for agent wishes [A4](agent-wishes.md#a4-api-lookup-from-the-cli) and
[S3](agent-wishes.md#s3-declaration-comments-as-the-single-source), together with the remaining
habits of [A2](agent-wishes.md#a2-diagnostics-for-pascal-and-ada-habits). Status: **planned, not
started.** Nothing here is implemented; the user-facing contract moves to
[`docs/pascal/program-structure/cli.md`](../pascal/program-structure/cli.md) and
[`docs/pascal/tools/diagnostics.md`](../pascal/tools/diagnostics.md) once it is.

The FPAS language does not change. This is CLI, language-service and diagnostics work only.

## Goal

`fpas doc <Name>` prints the signature, the declaration's `//` documentation and the owning
handbook page for one name: an intrinsic `Std.*` routine, a routine of a `Std.*` unit written in
FPAS, an operation on a built-in type such as `string.TrimLeft`, or a public declaration of the
user's own project. `--report json` returns the same as one JSON document. Examples come from the
documentation where it contains one; otherwise the handbook link points to them. The wish asked
for an example in every answer, but writing examples into the Rust catalog would duplicate the
handbook. Humans see the same comments on hover; one comment, two readers.

Success means:

- Every intrinsic `Std.*` symbol, every public declaration of the bundled FPAS `Std.*` units and
  every entry of the native operation catalog can be looked up by name.
- A public declaration of the current project is found with the same name resolution as hover,
  and its output contains exactly the documentation hover shows.
- An unknown name exits with a coded error whose hint lists the closest names, so an agent can
  retry without reading files.
- The [compact language reference](../pascal/agent-reference.md) tells authors to use `fpas doc`
  instead of reading `lib/api/` or Rust sources.

Second goal (P6–P8): the forms the diagnostics handbook lists as
[not recognized yet](../pascal/tools/diagnostics.md#diagnostics-for-other-language-spellings) get
habit diagnostics.

## Current state

- **Intrinsic units.** [`export_intrinsic_std_api`](../../crates/fpas-sema/examples/export_intrinsic_std_api.rs)
  generates editor-only declarations under [`lib/api/Std/`](../../lib/api/Std) from the Rust
  registry and the `docs/pascal/std/` tables, with `//` Markdown before each declaration.
- **Std units written in FPAS.** `Std.Tui`, `Std.Http`, `Std.Version` and the units below
  `lib/Std/` are ordinary sources with declaration comments.
- **Built-in type operations.** [`NativeOperation`](../../crates/fpas-sema/src/std_registry/native/mod.rs)
  holds receiver, name, signature and a one-line `documentation` per entry. They have no
  declaration file; only the editor reaches them through
  [`intellisense/native.rs`](../../crates/fpas-language-service/src/intellisense/native.rs).
- **Declarations and comments.** The language service already has a
  [`WorkspaceSymbolIndex`](../../crates/fpas-language-service/src/symbols/index.rs) with
  `find_qualified` / `find_unqualified`, [`public_signatures`](../../crates/fpas-language-service/src/public_signatures.rs),
  and [`preceding_documentation`](../../crates/fpas-language-service/src/documentation.rs), the
  extractor hover uses.
- **CLI.** Subcommands are selected in [`cli_input/mode.rs`](../../crates/fpas-cli/src/cli_input/mode.rs);
  machine-readable output elsewhere uses `--report json` (`init`, `test`).

## Design

### Command

```text
fpas doc <Name> [<path>] [--report json]
```

- `<path>` selects the project like `fpas check`: a `.fpas`, `.fpasprj` or `.fpasworkspace`; without
  it the current directory is discovered. With no project found, only `Std.*` and built-in
  operations are searched; this is not an error.
- Names are case-insensitive, like FPAS identifiers.
- Output is read-only and needs no compiled units to be written. If a project does not check,
  lookup still works on its parsed declarations, as hover does.

### Name forms

| Form | Example | Source |
| --- | --- | --- |
| Unit | `Std.Fs` | the unit's own comment and its public declarations as a list |
| Unit member | `Std.Fs.ReadText`, `Std.Tui.Run` | `lib/api/Std/` or `lib/Std/` declaration |
| Built-in operation | `string.TrimLeft`, `array.Push`, `dict.Keys`, `Option.Map`, `Result.MapError` | native catalog |
| Built-in operation, any receiver | `TrimLeft` | native catalog, all receivers |
| Project declaration | `App.Model.Total`, `Total`, `TPoint.Distance` | language-service symbol index |

Receiver names are the FPAS type spellings: `string`, `array`, `dict`, `Option`, `Result`. A
factory written on the type name (`string.…`, `array.…`) is listed beside the instance operation
of the same name, each marked with its kind. An unqualified name that matches several
declarations prints all of them, unit-qualified, instead of guessing.

### Output

Text, one block per match:

```text
function Std.Fs.ReadText(Path: string): result of (string, string)
  Reads UTF-8 text.
  see docs/pascal/std/host/fs.md
```

JSON (`--report json`), one document:

```json
{"query":"string.TrimLeft","matches":[{"kind":"operation","receiver":"string",
  "name":"TrimLeft","signature":"function TrimLeft(): string",
  "documentation":"Remove leading whitespace","handbook":"docs/pascal/language/types/string/case-trim.md",
  "location":null}]}
```

`location` holds path, line and column for source declarations and is `null` for catalog
entries. `handbook` is the owning page under `docs/pascal/`, when one is known. No match: exit
code `1`, a coded diagnostic with the up to five closest names (edit distance on the lowercase
name) in its hint, and in JSON an empty `matches` array plus the diagnostic.

### Where the logic lives

- New `fpas-language-service/src/api_lookup/`: query parsing, resolution over the three sources
  and the result type. It reuses the symbol index, `public_signatures`,
  `preceding_documentation` and the native catalog; nothing is duplicated in `fpas-cli`. A later
  `fpas query` ([N1–N4](agent-wishes.md#code-navigation)) can sit next to it.
- New `fpas-cli/src/cli_doc/`: argument handling, project discovery through the existing helpers,
  text and JSON rendering.
- The native catalog gains the handbook page per receiver, so the CLI and hover can link it.
  Whether its one-line `documentation` is enough for agents is checked in P5 against real
  authoring tasks; if not, the entries get longer descriptions there, still in the catalog.

## Habit diagnostics still missing

These extend the [habit table](../pascal/tools/diagnostics.md#diagnostics-for-other-language-spellings)
with the existing `ForeignHabit` builder and, where the replacement is complete source text, a
[fix](../pascal/tools/diagnostics.md#fixes).

| Habit | Origin | FPAS form | Stage | Fix |
| --- | --- | --- | --- | --- |
| `with Ada.Text_IO; use Ada.Text_IO;` before a program or unit | Ada | `uses Std.Console as Console;` | parser, `foreign/units.rs` | review |
| `package Geometry is … end Geometry;`, `package body Geometry is … end Geometry;` | Ada | `unit Geometry; … end unit;` with exports marked `public` | parser, `foreign/units.rs` | — |
| `task Worker;`, `task type Worker is … end Worker;`, `task body …` | Ada | `go Work()` inside `scope … end scope;`, `Std.Tasks` | parser, new `foreign/tasks.rs` | — |
| `begin … exception when others => … end;` | Ada | `Result of (T, E)` handled with `case` or `try Expr` | parser, `foreign/control_flow.rs` | — |
| `raise Constraint_Error;`, `raise Exception.Create('…');` | Ada, Delphi | `return Error('…');`, or `panic('…')` for a bug | parser or sema | — |
| `S[1]` as the first character, `S[Length(S)]` | Pascal | zero-based: `S[0]`, `S[S.Length() - 1]` | runtime hint | — |

One-based string indexing is valid FPAS and cannot be told apart at compile time. Instead, the
runtime index-out-of-range error for a string or array indexed at exactly its length adds the
hint "strings and arrays start at 0; the last index is `Length() - 1`". No static diagnostic is
added for it.

## Work packages

| Package | Content | Depends on |
| --- | --- | --- |
| P1 | `api_lookup` in the language service: query parsing, native catalog and `Std.*` sources, result type, closest-name suggestions | — |
| P2 | Project declarations through the symbol index, including record methods and units of the workspace | P1 |
| P3 | `fpas doc` command, help text, text and JSON output, error code for no match | P1 |
| P4 | Handbook page per native receiver and per `Std.*` unit in the output; hover links the same page | P1 |
| P5 | Docs: CLI reference, agent reference (replace "read `lib/api/`"), `fpas-authoring` skill, `Std` touchpoints; blind authoring check with `fpas doc` only | P2–P4 |
| P6 | Ada `with` / `package` units | — |
| P7 | Ada tasks, exception handlers and `raise` | — |
| P8 | Runtime hint for an index equal to the length | — |
| P9 | Habit table and agent reference rows for P6–P8; remove this plan | P5–P8 |

P6–P8 are independent of P1–P5 and of each other and can fill gaps between them.

## Tests

- P1: unit tests per name form, case-insensitivity, factory and instance operation of the same
  name, ambiguous unqualified names, suggestions for a misspelled name.
- P1: a test that every `intrinsic_std_symbols` entry and every `native_operations` entry resolves,
  so a new catalog entry cannot be missed.
- P2: hover and `fpas doc` return the same documentation for the same declaration, including a
  project on disk with a record method and a qualified import.
- P3: CLI tests for text and JSON output, no project, a project that does not check, no match and
  its exit code, and the stdout/stderr split.
- P6–P7: positive, negative and recovery tests beside the existing `foreign_*` tests; fixes are
  applied and rechecked with the shared helper.
- P8: a VM test for the hint on a string and an array, and none for other out-of-range indexes.
- An FPAS regression fixture under `tests/` with Ada-style sources for P6–P7.

## Open questions

- Should a unit query (`fpas doc Std.Fs`) list every public declaration with its first comment
  line, or only names and signatures?
- Should `fpas doc` also accept keywords (`fpas doc scope`, `fpas doc case`) and point to the
  language handbook page, or stay limited to declarations?
