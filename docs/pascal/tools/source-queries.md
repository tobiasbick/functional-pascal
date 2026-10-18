# Source queries

`fpas query` reads saved sources through the editor's language service. Each
navigation request writes one UTF-8 JSON object and a newline to stdout, including
invalid requests. Stderr is reserved for a final output-write failure. Queries
parse and analyze sources without compiling bytecode, executing FPAS, or writing
source/build artifacts. Private worker IPC files are removed when the request ends.

```sh
fpas query refs src/main.fpas:12:5
fpas query refs src/math.fpas:4:17 --context suite.fpasworkspace --limit 200
fpas query definition src/main.fpas:12:5 --context app.fpasprj
fpas query type src/main.fpas:12:5 --std-lib ./lib
```

Use `fpas query --help` or the operation's `--help` for examples. Canonical names
are `refs`, `definition`, and `type`; there are no aliases. Each takes exactly one
`.fpas` file and one position. Quote the complete position when its path contains
spaces. The last two colon-separated components are positive line and column
numbers, so Windows drive letters are preserved. Positions and half-open output
ranges count one-based Unicode scalars: an emoji or tab counts as one column.
LF, CRLF and bare CR line endings use the same coordinates. The BOM remains part
of the exact source bytes and coordinate conversion. Invalid positions are
rejected; token ends, EOF, comments and strings do not select a preceding name.

## Context and bounds

Without `--context`, nearest source ownership determines the project/workspace.
A loose source has an explicitly limited scope: that file and declared imports
available through the selected source standard library. An explicit context must
be a `.fpasprj` or `.fpasworkspace` manifest; directories are rejected. Projects
include selected sources and reachable dependencies; workspaces include all
member projects and dependencies. Test programs are separate analysis roots.
Ambiguous ownership, invalid manifests and out-of-context sources are errors.
Consumers outside this scope are not discovered. Resolution preserves each
consumer's import visibility and library export policy. Unused Std implementation
bodies are not searched for uses of a user declaration.

`--std-lib <dir>` replaces the complete source standard library containing
`stdlib.fpasprj`; by default the installed adjacent library is used.
`--timeout <seconds>` sets a positive whole-invocation deadline (default 30).
The parent requests cancellation then allows at most 250 ms before terminating
and reaping its isolated worker process tree. OS containment and uninterruptible
OS/final-output limitations are the same as [batch debugging](debugger-batch.md).

`refs --limit <count>` bounds retained uses during traversal: default 1,000,
maximum 10,000. Reaching the bound yields explicit incomplete coverage when more
uses exist; an unknown total stays null. There is no declaration inclusion flag:
the declaration always appears separately under `target`.

Inputs share immutable request snapshots, including manifests and file selection.
The ceilings are 4,096 canonical participating files, 8 MiB per file and 64 MiB
in total. Excluded sources do not count. Discovery also bounds retained paths
and directories; exceeding discovery capacity is incomplete. Source and manifest
SHA-256 hashes cover exact bytes. Before publication the worker rechecks bytes,
file membership and lexical path identities. Changes yield `source_changed`,
without retrying or combining revisions. This is a checked snapshot rather than
an atomic filesystem transaction.

## Questions

- `refs` returns resolved uses of one declaration, excluding its defining name.
  Case differences, shadowing, aliases, fields, patterns and named labels follow
  semantic identity. An unused resolved declaration succeeds with empty matches.
- `definition` returns the defining identifier as one match and under
  `target.declaration`, including when the request selects that declaration.
  Callable values resolve to their own declaration. Generic uses retain the
  original source declaration. Imported aliases retain their local identity.
- `type` returns a checked static type and its selected source location. It uses
  original compiler AST metadata, including generic substitutions and chain
  prefixes. It never parses hover text or evaluates an expression.

Type selection chooses the innermost checked expression, with deeper expressions
winning equal spans. An argument selects its own type; an operator selects its
containing expression. A routine name used as a callee selects the call result;
a bare routine value has a callable type. In `Items[0].Name.Length()`, `Items`
selects the array type, `Name` the field type, and `Length` the call result. The
index expression has its own type. Value declaration names, parameters, loop and
pattern bindings, and named actual/constructor/pattern labels select their checked
receiving type. Labels take precedence over the containing call. Comments are
excluded; every part of a string literal selects the string type. Whitespace
inside an expression can select its enclosing expression. Type declarations,
unit/import names and procedure statements have no selected value type. Missing
metadata, unresolved type holes and error types give incomplete coverage.

The `type` object contains canonical compiler `display`, semantic `kind`, selected
`location`, and a physical nominal `declaration` or null. Categories are `integer`,
`real`, `boolean`, `string`, `array`, `dict`, `record`, `enum`, `option`, `result`,
`callable`, `task`, `channel`, `subrange`, `distinct`, and `generic_parameter`.
Distinct types preserve their own identity. Internal recursive type structures
are not serialized.

A built-in without physical source has `origin: builtin` and a null declaration.
`definition` reports `not_found` with `no_source_definition`; `refs` reports
`incomplete` with `unsupported_target`. Source-backed Std API declarations remain
eligible. A checked built-in call result can still have a type.

## JSON contract

Schema version 1 uses these required fields:

| Field | Content |
| --- | --- |
| `schema_version`, `operation` | `1` and `refs`, `definition`, `type`, or null for an unidentified request. |
| `outcome`, `exit_code` | Outcome and matching process status from the table below. |
| `request` | Requested `file`, `line`, `column`; unavailable parts are null. |
| `context` | Scope `kind`, selected `manifest`, sorted `projects`, and absolute invocation directory `path_base`. |
| `target` | Name, qualified name, kind, origin, and declaration location; null when unresolved. |
| `matches` | Sorted, deduplicated use/definition locations; empty for type requests. |
| `type` | Checked type object for type requests; otherwise null. |
| `diagnostics` | Original attributed FP records and named query diagnostics. |
| `coverage` | `complete`, omission `reasons`, hashed `files`, `returned_count`, known `total_count` or null. |
| `limits` | Effective timeout, result count and source/JSON/text ceilings. |

Paths inside the invocation directory are relative to it; other paths are
absolute. All use `/` separators. Locations contain `path`, a half-open `range`
with scalar `start`/`end`, a zero-based UTF-8 `byte_span` (`start`, `length`), and
one line `preview` (`text`, one-based `column_start`, `truncated`). Previews retain
at most 160 scalars and come from the same snapshot as the location. Matches are
ordered by reported path and byte offset. Canonical identity and span deduplicate
physical sources, preserving declaration spelling.

The complete escaped JSON including newline is at most 8 MiB. Type/signature and
diagnostic fields retain at most 4,096 scalars and carry explicit clipping markers.
Text clipping alone does not reduce semantic coverage. Omitted matches, type,
identity or source coverage do. The primary diagnostic and useful partial data
survive result-budget clipping.

| Exit | Outcome and meaning |
| --- | --- |
| 0 | `ok`: complete success, including an unused declaration. |
| 1 | `not_found`: complete scope without the selected target/type; or `error`: request/setup/read failure. |
| 3 | `incomplete`: analysis errors, unsupported target, limits, source changes; or `error`: worker/internal failure. |

Parse/type recovery may retain proven matches and original diagnostics, with
incomplete coverage. Missing analysis is never a complete empty result. Forced
worker failure can leave coverage unavailable and no target/matches. Query commands
have no runtime-error exit 2. A final stdout failure may leave a partial document;
its process status is nonzero.

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

## Rename failures

Rename retains its separate edit/publication schema, `--project` context option
and stderr diagnostic records. Argument, source, semantic, publication and output
failures exit 1. A failed rename has no success document. If stdout fails after
publication, edits have already been applied; inspect saved sources before retrying.
