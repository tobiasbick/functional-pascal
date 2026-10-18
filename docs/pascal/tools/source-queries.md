# Source queries

`fpas query` answers one source-navigation question with one JSON document on stdout.
It uses the same language service as the editor, reads saved sources, and creates no
compiled units, program artifacts, or source edits.

```sh
fpas query refs src/main.fpas:12:5
fpas query refs src/main.fpas:12:5 --include-declaration
fpas query def src/main.fpas:12:5
fpas query type src/main.fpas:12:5
fpas query refs src/math.fpas:4:17 --project suite.fpasworkspace
fpas query type src/main.fpas:12:5 --std-lib ./lib
```

Use `fpas query --help` or `fpas query refs|def|type --help` for focused help.
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

Query coverage follows the shared language service; a built-in operation without a source declaration can have
an expression type and no definition or references.

## JSON contract

Schema version 1 always contains `schemaVersion`, `query`, `position`, and `matches`.
`query` is `refs`, `def`, or `type`. `position` contains an absolute source `path`,
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

Argument, position, source-read, project-load, semantic-analysis, and output-write
failures exit with status 1. Before writing a success document, query execution
rejects erroneous input analysis. Execution failures use the existing diagnostic
JSON records on stderr; argument-parser failures use ordinary CLI diagnostics.
No success document is written for a failed query. An output failure can leave a
partial document and must be detected through its nonzero exit status.
