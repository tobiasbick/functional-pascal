# 01: decided query contract

These are implementation requirements selected by the agent under the user's
explicit delegation. They are not current language/tool documentation.

## Commands

```text
fpas query refs src/main.fpas:12:5
fpas query definition src/main.fpas:12:5 --context app.fpasprj
fpas query type src/main.fpas:12:5 --context suite.fpasworkspace
```

Canonical operation names are `refs`, `definition` and `type`, without aliases.
Each request has exactly one saved `.fpas` file and one position. Parse the two
positive numeric suffixes from the right, preserving spaces and Windows drives.
All input paths are relative to the invocation directory unless absolute.

| Option | Contract |
| --- | --- |
| `--context <manifest>` | Optional `.fpasprj` or `.fpasworkspace`; selects the complete search/analysis boundary. No directory or second source positional argument. |
| `--std-lib <dir>` | Existing complete source-library replacement; default is the installed adjacent library. |
| `--timeout <seconds>` | Positive whole-invocation deadline; default 30 seconds. |
| `--limit <count>` | `refs` only: positive maximum returned uses, default 1,000, ceiling 10,000. |

Reject duplicate singleton options, unknown options, extra arguments and options
for another operation. No stdin input, prompts, editor overlay, server session,
compiled-image target, expression-text argument, JSONL, text result or `--report`
switch. Help/version retain ordinary CLI output, with focused help and examples.

Lines/columns match FPAS diagnostics: one-based Unicode-scalar positions, with a
tab counting as one scalar. Convert against the exact loaded source. LF, CRLF,
bare CR, BOM and supplementary Unicode need consistent handling. A position past
line content plus its end boundary is invalid; do not clamp or snap it. A valid
position at token end, in whitespace/comment/string text, or at EOF does not
silently select the preceding identifier. Operation-specific type selection is
specified in [04](04-type.md).

## Context, identity and source consistency

Without `--context`, discover the nearest owning project/workspace for the source
using the existing ownership rules. Never recursively scan the current directory
or silently select an unrelated manifest. A source without an owner is a loose
file: inspect it and its declared source imports available through the selected
standard library, and label that limited scope explicitly.

An explicit project includes its manifest-selected own sources and reachable
source dependencies. An explicit workspace includes all declared member projects
and their reachable source dependencies; no sole-program restriction. Test
projects are valid contexts, including their manifest-selected test programs.
Use source visibility/export rules when resolving from each consuming file.
Do not claim to discover consumers outside this context or reverse dependencies
outside a loose-file query. Do not search unused standard-library implementation
bodies for uses of a user declaration merely because the library is installed.

Ambiguous ownership, a broken selected manifest and an out-of-context target are
request/setup errors (exit 1); never broaden explicit scope, choose the first project or fall back to a
loose file. Directory catalog truncation and ignored/unreadable selected sources
must not become a complete empty result. Treat valid program/test entry sources
as separate analysis roots when a project selects several programs.

Symbol identity is the resolved declaration and its source snapshot, not the
case-insensitive text of its name. Preserve shadowing, imports, aliases,
visibility, record/enum members, patterns, generics and named parameter labels.
Do not merge unrelated fields/routines because their spellings match.

Load each participating source/manifest into an immutable request snapshot and
use it for resolution, positions, snippets and semantic analysis. Report SHA-256
hashes of the exact file bytes. Check the source set/manifests again before publication; a detected
change yields `incomplete` with `source_changed`, without automatic retries or
mixed-revision coordinates. This is a checked snapshot, not an atomic filesystem
transaction. No source/build writes or program execution are allowed. Private
worker IPC scratch files are the only tool-owned writes and are cleaned up.

## Operations

**Refs (N1):** select an identifier token by declaration or use. Return every
resolved use of that exact declaration in scope, excluding its defining
selection span. Include recursion and semantic labels where they refer to the
same declaration; exclude strings/comments, local shadows and unrelated members.
An unused but resolved declaration returns an empty `matches` list successfully.
The defining location is available once under `target`; no inclusion toggle.

**Definition (N2):** resolve that same identifier and return its defining name
range, including when the position is already the declaration. One target is
expected in FPAS. Ambiguity is an explicit diagnostic, never an arbitrary first
match. Follow lexical/import/alias identity, not a guessed type declaration or a
callable value's possible runtime destination. Details are in [03](03-definition.md).

**Type (N3):** return the checked static type at the selected source construct,
with exact selected range and canonical type display/category. Expression and
binding rules are in [04](04-type.md). No compilation to bytecode, execution,
hover-text parsing, runtime-type guessing or silent `dynamic` placeholder.

Built-in operations without a physical declaration must not gain a fake source
range. A resolved built-in may have `origin: builtin`, a null declaration and
its canonical API name for `fpas doc`. N2 then reports `not_found` with reason
`no_source_definition`. N1 delivers references for source-backed declarations;
built-in operations/types without a physical declaration are outside its first
scope and return `incomplete`/`unsupported_target`. Do not scan matching spellings
or report a complete empty list for them. Real source/API declarations for native
standard-library routines remain eligible. N3 can inspect a built-in call's
checked result type without needing a physical type declaration.

## One versioned JSON result

Every ordinary query writes exactly one UTF-8 JSON object followed by a newline
to stdout. Stderr stays empty except a final publication failure. The normal
exceptions for external kills, OOM/crashes and blocked final stdout apply.

| Required field | Content |
| --- | --- |
| `schema_version` | Integer `1`. |
| `operation` | `refs`, `definition` or `type`; null if the request cannot identify one. |
| `outcome` | `ok`, `not_found`, `incomplete` or `error`. |
| `exit_code` | Process exit code from the table below. |
| `request` | Requested file/line/column; unavailable components are null on invalid input. |
| `context` | Scope kind, selected manifest, project identities and absolute invocation directory used as the output path base. |
| `target` | Resolved name, qualified name, kind, origin and declaration location; null if unresolved. |
| `matches` | Ordered use/definition locations; empty for type requests. |
| `type` | Checked type object for N3, otherwise null. |
| `diagnostics` | Original structured FP records and/or named query diagnostics. |
| `coverage` | `complete`, omission reasons, source/manifest hashes, returned count and known total or null. |
| `limits` | Effective timeout, result-count, source and JSON/text budgets. |

Report file paths as invocation-relative when inside that directory and absolute
otherwise, with `/` separators. No URI encoding or path-shortening flags. A
location has `path`, a half-open `range` with one-based scalar `start`/`end`, and
`byte_span` with zero-based `start` and `length` in that exact source snapshot.
Attach one bounded line preview (`text`, `column_start`, `truncated`) to matches
and the target declaration. Do not put display-cell or UTF-16 columns into these
ranges. Source snippets and structured diagnostic locations retain their own
source provenance; never resolve offsets against another file.

Sort locations by reported path and byte start, deduplicate by canonical source
identity and exact span. Preserve declaration spelling. Project/source listings
are sorted too. Do not emit process IDs, elapsed timing or random/session IDs as
part of a successful semantic answer.

| Exit | Meaning |
| --- | --- |
| 0 | Complete successful query, including a resolved declaration with zero uses. |
| 1 | Invalid request, setup/read failure, or a complete `not_found` result. |
| 3 | Incomplete/unsupported analysis, ambiguous symbol resolution, a resource/deadline bound, source changes or worker/internal failure. |

`ok` uses exit 0, `not_found` uses exit 1, and `incomplete` uses exit 3.
`error` uses exit 1 for request/setup failures and exit 3 for worker/internal failures.
There is no runtime-error exit 2: queries do not run FPAS. A clean scope with no
selected symbol/type yields `not_found`; missing/degraded analysis yields
`incomplete`, not `not_found`. Parse/type errors may retain useful proven matches
with original diagnostics but force incomplete coverage and exit 3. Preserve
primary diagnostics and available results even when optional text is clipped.

## Budgets and lifecycle

Fixed input ceilings: 4,096 participating files, 8 MiB per file and 64 MiB of
text in total, counting selected sources and manifests together after canonical
file deduplication. Enforce these while discovering/reading, before constructing
unbounded source lists or parsing oversized text.
The complete escaped JSON including newline is at most 8 MiB. Type/signature and
diagnostic text is at most 4,096 Unicode scalars per field; preview text is at
most 160 scalars. Preview/diagnostic text clipping has an explicit marker; it does
not alone mean missing semantic matches. Losing a requested type or any match,
source coverage, or required identity makes the answer incomplete. Never turn a
valid result into another primary error merely to fit the JSON budget.

The parent owns an isolated query worker and the deadline, including discovery,
reads, analysis, consistency checks and result preparation. Request cooperative
cancellation, then allow at most 250 ms before terminating/reaping its owned
process tree. Reuse D3/test-runner process containment; extract genuinely shared
hosting code where both production callers need it instead of copying the debug
protocol or introducing a generic service framework.

A result limit bounds collection/retention, not just the final array. Stop with
explicit partial coverage when a budget is reached; known totals remain null if
counting stopped early. A forced kill may leave no query snapshot; publish a
complete error envelope with unavailable coverage, never stale prior matches.
Keep private argv transport budgets separate from source/result budgets so JSON
escaping cannot reject an otherwise valid CLI request. Platform containment and
uninterruptible OS/final-output limitations match the batch-debugger contract.
