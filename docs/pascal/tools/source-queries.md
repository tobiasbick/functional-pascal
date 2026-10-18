# Source queries

`fpas query` answers one source-navigation question with one JSON document on stdout.
It uses the same language service as the editor and reads saved sources. `refs`,
`def`, and `type` are read-only. `rename` applies validated source edits unless
`--dry-run` is present. None of these commands creates compiled units or program artifacts.

```sh
fpas query refs src/main.fpas:12:5
fpas query refs src/main.fpas:12:5 --include-declaration
fpas query def src/main.fpas:12:5
fpas query type src/main.fpas:12:5
fpas query rename src/main.fpas:12:5 Total --dry-run
fpas query rename src/main.fpas:12:5 Total
fpas query refs src/math.fpas:4:17 --project suite.fpasworkspace
fpas query type src/main.fpas:12:5 --std-lib ./lib
```

Use `fpas query --help` or `fpas query refs|def|type|rename --help` for focused help.
Paths may be relative to the working directory or absolute, including Windows
drive letters. Quote the entire position argument when its path contains spaces.
The last two colon-separated components are the line and column, both starting
at 1. Columns count Unicode scalar values, like compiler diagnostics: an emoji
counts as one column. Tabs also count as one character, independently of display width.

The source's enclosing project is discovered automatically. `--project <path>`
selects a project manifest, workspace manifest, or project folder explicitly;
the source must belong to a project in that context. A workspace lets references
include dependent member projects known to the language service. Without a
manifest, queries work on the loose source and its standard-library context.
`--std-lib <directory>` selects a complete source standard library containing
`stdlib.fpasprj`, with the same default as `fpas doc`.

## Questions

- `refs` finds uses of the resolved symbol, respecting identifier case, lexical
  shadowing, import aliases, and project visibility. Strings and comments are
  not references. `--include-declaration` also returns the defining declaration.
  Results are sorted by source path and offset, with duplicates removed.
- `def` finds the resolved declaration, with the range of its name.
- `type` returns the compiler's checked type of the smallest expression containing
  the position. This includes literals, inferred values, calls, generic results,
  fields, indices, operators, closures, and expression branches. Inside a chain,
  a completed prefix can supply the type before later operations. Place the
  position on an operator or opening delimiter to select a larger expression
  rather than one of its operands. Declaration names are not expressions.
- `rename <new-name>` renames the resolved declaration and its references across
  the selected project context. It follows aliases and case-insensitive references,
  preserves shadowed declarations, and leaves strings and comments untouched.
  `--dry-run` validates the same edits without writing sources.

Query coverage follows the shared language service; a built-in operation without a source declaration can have
an expression type and no definition or references.

## JSON contract

Schema version 1 contains `schemaVersion`, `query`, and `position`.
`refs`, `def`, and `type` also contain `matches`. `position` contains an absolute source `path`,
`line`, and `column`. All paths use forward slashes. Every result range has
inclusive `start` and exclusive `end`, each with one-based `line` and `column`.
Ranges are translated from the exact snapshots used by the language service.

A reference result looks like this:

```json
{
  "schemaVersion": 1,
  "query": "refs",
  "position": {"path": "/project/main.fpas", "line": 4, "column": 9},
  "matches": [
    {
      "path": "/project/main.fpas",
      "range": {"start": {"line": 5, "column": 11}, "end": {"line": 5, "column": 16}},
      "is_declaration": false
    }
  ]
}
```

Definition matches contain `path` and `range`. Type matches contain `type` (the
FPAS type spelling) and `range` in the input source. `matches` is empty when
there is no symbol or typed expression, or a resolved symbol has no references.
An empty result is successful and exits with status 0.

## Safe rename

```sh
fpas query rename src/math.fpas:4:17 Compute --project suite.fpasworkspace --dry-run
fpas query rename src/math.fpas:4:17 Compute --project suite.fpasworkspace
```

Rename returns `new_name`, `applied`, and `edits` instead of `matches`:

```json
{
  "schemaVersion": 1,
  "query": "rename",
  "position": {"path": "/project/main.fpas", "line": 4, "column": 9},
  "new_name": "Total",
  "applied": false,
  "edits": [
    {
      "path": "/project/main.fpas",
      "range": {"start": {"line": 4, "column": 9}, "end": {"line": 4, "column": 14}},
      "old_text": "Count",
      "new_text": "Total"
    }
  ]
}
```

Edits are sorted by source path and descending source offset within each file.
Ranges refer to the saved sources before the rename. `applied` is false for
`--dry-run` and true after successful application. References already spelled
exactly like `new_name` need no edit. When the selected identifier and all its
references already use that spelling, `edits` is empty and the command succeeds.
A position without a renameable symbol is an error.

The shared language service rejects invalid identifiers, keywords, conflicts with
declarations or import aliases, and renames that would change lexical binding.
Program and unit names require file or manifest changes and cannot be renamed.
Generated intrinsic APIs, the selected standard library, and sources outside the
selected root are protected. The root is the discovered project directory, the
explicit workspace directory, or the selected folder; containment is checked
against physical paths so links cannot redirect edits outside that root. Select
the containing workspace to rename a shared library and its known consumers together.
Consumers outside the selected context are not discovered automatically.

Rename checks every known project source, including consumers, before and after
applying the proposed edits in memory. Existing source errors also prevent rename.
Both modes reject a project source that changes during analysis. Application stages every
replacement before publishing any file, preserves file permissions and untouched
UTF-8 bytes (including line endings), and checks the saved sources again before
publication. Ordinary publication failures trigger restoration of files already
replaced; restoration failures identify the affected paths on stderr. Concurrent
writers and process termination are not covered by a filesystem transaction.

## Failures

Argument, position, source-read, project-load, semantic-analysis, and output-write
failures exit with status 1. Before writing a success document, query execution
rejects erroneous input analysis. Execution failures use the existing diagnostic
JSON records on stderr; argument-parser failures use ordinary CLI diagnostics.
No success document is written for a failed query. An output failure can leave a
partial document and must be detected through its nonzero exit status. If stdout
fails after rename publication, source edits have already been applied; inspect
the saved sources before retrying.
