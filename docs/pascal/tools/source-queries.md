# Source queries

`fpas query` reads saved sources through the editor's language service. Each
navigation request writes one UTF-8 JSON object and a newline to stdout, including
invalid requests. Stderr is reserved for a final output-write failure. Queries
parse and analyze sources without compiling bytecode, executing FPAS, or writing
source/build artifacts. Private worker IPC files are removed when the request ends.

```sh
fpas query find Clamp --context app.fpasprj
fpas query definition --symbol App.Math.Clamp --context app.fpasprj
fpas query type --target-ref clamp.target.json
fpas query inspect --symbol App.Math.Clamp --context app.fpasprj
fpas query callers --symbol App.Math.Clamp --context app.fpasprj
fpas query callees --target-ref clamp.target.json
fpas query refs src/main.fpas:12:5
fpas query refs src/math.fpas:4:17 --context suite.fpasworkspace --limit 200
fpas query definition src/main.fpas:12:5 --context app.fpasprj
fpas query type src/main.fpas:12:5 --std-lib ./lib
```

Use `fpas query --help` or the operation's `--help` for examples. Canonical names
are `find`, `inspect`, `refs`, `definition`, `type`, `callers`, and `callees`; there are no aliases. `find` takes
nonempty search text and an explicit manifest context. The other read operations
accept exactly one source position, `--symbol <name>`, or `--target-ref <file>`.
Name selection requires a manifest and matches exact short or qualified names,
case-insensitively. Ambiguous names return candidates without choosing a target.
File-local import aliases require a position or reference. Quote a position when its path contains
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

Save the exact `target_ref` object from a checked declaration to reuse it through
`--target-ref`. It carries its context and absolute source identity, independently
of the new working directory. Any explicit context/library override must still
match. Changes to participating bytes, manifests, membership or the shared
analysis build invalidate it with `stale_target`; malformed references are
`invalid_request`. References are capped at 8 MiB and never authorize writes.

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

`find --limit <count>` defaults to 100 and permits at most 10,000 symbols. The
collector retains only the best matches during traversal. Exact short/qualified
matches precede short prefixes, short substrings, qualified prefixes and qualified
substrings. Qualified name, project, file and declaration position break ties.
Clipped results retain the known total when discovery completed. A retained
reference can still be valid even though other matches were omitted.

Inputs share immutable request snapshots, including manifests and file selection.
The ceilings are 4,096 canonical participating files, 8 MiB per file and 64 MiB
in total. Excluded sources do not count. Discovery also bounds retained paths
and directories; exceeding discovery capacity is incomplete. Source and manifest
SHA-256 hashes cover exact bytes. Before publication the worker rechecks bytes,
file membership and lexical path identities. Changes yield `source_changed`,
without retrying or combining revisions. This is a checked snapshot rather than
an atomic filesystem transaction.

## Questions

- `find` searches owned declarations (including private declarations and locals),
  eligible imported dependency exports and compiler APIs. It reports origin,
  visibility, project, checking state, signature and available documentation.
  Syntax candidates never prove uniqueness. An empty complete search succeeds.
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
- `inspect` returns the signature and documentation, the physical FPAS body when
  available, then directly named types from parameters, result, receiver and
  explicit local annotations. Compiler bindings preserve aliases and shadowing.
- `callers` returns proven incoming calls and related callable uses across the
  selected context. Unresolved calls remain conservatively relevant even when
  no proven edge reaches the selected declaration.
- `callees` returns relationships inside one selected body. Nested routines,
  closures, record-field defaults and retained static-bound declarations have
  separate owners. An empty completely analyzed body succeeds.

Call queries keep `direct_calls`, `callable_values`, `callback_arguments` and
`dynamic_calls` separate. Each entry carries the exact `location`, `containing`
owner, proven `target` or null, consumer `context`, `invocation` (`ordinary` or
`spawn`), and written arguments with their checked parameter indices. A callback
entry also identifies its receiving callee/parameter when known; passing it does
not prove that it executes. Record/enum construction and scalar conversions are
not routine edges. Native targets have compiler identities and no invented body.

`direct_edges_complete` reports analysis coverage independently of output limits.
`runtime_targets_complete` is false when relevant dynamic calls remain. A complete
direct inventory can therefore have an `incomplete` overall outcome while keeping
its verified declaration reference. Neither field claims coverage of implicit
compiler invocations such as resource cleanup. Source Std bodies participate when
selected; unrelated Std bodies and consumers outside the context are excluded.

`--limit` defaults to 1,000 and permits at most 10,000 entries **per collection**.
Each collection has independent returned/known-total/completeness fields, also
projected under `coverage.collections`. Ordering uses physical file, byte span,
relationship/target and consumer identity. Distinct consumer interpretations are
retained. A field-default call is recorded once in its declaration environment,
regardless of how many constructions or generic applications use that default.

Inspection follows types one hop. It does not expand callees, imported modules,
member types or supporting method bodies. Contracts remain in the signature;
nested routines remain part of the selected implementation. Generated intrinsic
API stubs provide declarations, never native implementations. Source-less
builtins retain their known signature and an explicit `no_source` omission.

`result.sections` carry their kind, reason, owner, origin, checking state, full
location/hash, documentation and exact continuous `excerpts`. Each excerpt has
its own location and original text, including line endings. Disjoint excerpts
remain separate. Construction retains at most 32 KiB of aggregate UTF-8 source
text and 12 supporting declarations, in signature/receiver/local priority.
`full_bytes`, `omitted_bytes` and `result.omissions` explain unavailable bodies,
deliberate scope exclusions and clipping. These omissions do not by themselves
make semantic analysis incomplete; unavailable compiler bindings do.

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

The `result.type` object contains canonical compiler `display`, semantic `kind`, selected
`location`, and a physical nominal `declaration` or null. Categories are `integer`,
`real`, `boolean`, `string`, `array`, `dict`, `record`, `enum`, `option`, `result`,
`callable`, `task`, `channel`, `subrange`, `distinct`, and `generic_parameter`.
Distinct types preserve their own identity. Internal recursive type structures
are not serialized.

`type --symbol` selects the declaration's value type. A named routine has its
callable type; a unit or type declaration has no value type. Source-less intrinsic
routines can have a compiler-known callable type with a null source location.

A built-in without physical source has `origin: builtin` and a null declaration.
`definition` reports `not_found` with `no_source_definition`; `refs` reports
`incomplete` with `unsupported_target`. Source-backed Std API declarations remain
eligible. A checked built-in call result can still have a type.

## JSON contract

Schema version 2 uses these required fields:

| Field | Content |
| --- | --- |
| `schema_version`, `operation` | `2` and `find`, `inspect`, `refs`, `definition`, `type`, `callers`, `callees`, or null for an unidentified request. |
| `outcome`, `exit_code` | Outcome and matching process status from the table below. |
| `request` | Tagged `selector`, operation `arguments` and effective `options`; an unavailable selector is null. |
| `context` | Scope `kind`, absolute reconstruction `input`, physical `root`, selected `standard_library`, sorted `projects`, `policy`, and invocation `path_base`. |
| `snapshot` | Shared analysis identity, complete input inventory and content-derived snapshot ID; null when unavailable. |
| `target`, `candidates` | Selected declaration and checked `target_ref`, or ambiguity candidates. An unavailable reference has an explicit reason. |
| `result` | Tagged payload: `symbols`, inspection `sections`/`omissions`, `locations`, `type`/`unavailable_reason`, or call collections and their two coverage flags. |
| `diagnostics` | Original attributed FP records and named query diagnostics. |
| `coverage` | `analysis_complete`, `reasons`, per-collection `returned`/`known_total`/`complete`, and textual `omissions`. |
| `limits` | Effective timeout, result count and source/JSON/text ceilings. |

Display paths inside the invocation directory are relative to it; other paths are
absolute. All use `/` separators. Locations contain `path`, a half-open `range`
with scalar `start`/`end`, a zero-based UTF-8 `byte_span` (`start`, `length`), and
one line `preview` (`text`, one-based `column_start`, `truncated`). Previews retain
at most 160 scalars and come from the same snapshot as the location. Matches are
ordered by reported path and byte offset. Canonical identity and span deduplicate
physical sources, preserving declaration spelling.

Snapshot identities use lossless absolute paths and SHA-256 over the complete
manifest/source inventory, lexical path bindings and discovery membership.
They also include the shared analysis-core build identity and library selection.
Result limits do not change a declaration's identity. A clipped locations list
can retain a checked reference when its complete input scope was verified;
incomplete analysis or changed inputs cannot. CLI and LSP share these identities.

The complete escaped JSON including newline is at most 8 MiB. Type/signature and
diagnostic fields retain at most 4,096 scalars and carry explicit clipping markers.
Text clipping alone does not reduce semantic coverage. Omitted locations reduce
their collection coverage. Identities and edit bytes are never truncated. If
required fields cannot fit, the report replaces the payload with an explicit
`json_limit` failure rather than emitting a partial identity.

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

## Checked source changes

```sh
fpas refactor rename src/math.fpas:4:17 --name Compute --context suite.fpasworkspace --out rename.plan.json
fpas refactor apply --plan rename.plan.json
```

The [refactoring commands](refactoring.md) use the same checked context and
schema-2 envelope. Preparation returns exact edits and complete before/after
inventories without changing source. Apply revalidates every input and regenerates
the recipe before publication. A complete matching after state returns
`already_applied`; partial application or unrelated input changes return
`stale_plan`. Extraction and import organization use the same flow.
