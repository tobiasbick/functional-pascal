# Diagnostics

Compiler diagnostics identify a problem by code and severity, with a source range
when one exists. They can also carry explanation, help, expected/found data,
related locations and proposed corrections. Compilation and project checking use
the same diagnostic model.

## Output and exit status

Normal CLI output writes diagnostics to stderr with the header
`path(line,column): error FPnnnn: message` or the corresponding warning form,
followed by a numbered source excerpt, caret/range marker and available help,
notes and correction titles. Long lines are cropped around the selected range.
Tabs use four-column stops; grapheme clusters, combining marks and wide characters
are accounted for when aligning markers. Terminal-specific glyph widths may vary.
Control characters are escaped. Color is automatic for an interactive stderr;
redirected output, `NO_COLOR` and `TERM=dumb` disable it.
Successful compilation writes `Wrote path.dll` to stdout.

Use `--diagnostics json` for automation. It writes one stdout object containing
`success`, `output`, and `diagnostics`; each diagnostic contains `code`, `message`,
`severity`, `location`, `explanation`, `help`, `helpLink`, `expectation`, `related`
and `fixes`. The output path is null when compilation fails. Optional data is null
or omitted when unavailable; related locations and fixes are arrays. Check/query
diagnostics retain these fields and additionally identify the owning project.

Source locations contain `path`, one-based UTF-16 `line`/`column`, UTF-16 `length`,
exclusive `endLine`/`endColumn` and zero-based UTF-16 `offset`. Tabs count as one
UTF-16 code unit in the data. Input line endings and offsets are preserved.
EOF ranges may be empty. Non-source failures have no location; command-line,
I/O and reference setup failures do not invent a source token. `inputPath` can
identify an affected input/output file without claiming source coordinates.

`expectation` contains a `kind`, an `expected` string array and a `found` string.
It is supplied for parser expectations, type conversions, invalid numeric values
and supported correction hints. Consumers must not parse the primary message to recover that data.
Each related note has a message and an optional location.

Each fix contains `title`, `applicability` (`automatic` or `review`) and `edits`.
An edit specifies its exact `location`, `expectedText` and `newText`; an empty
range inserts text. Automatic identifies a direct local correction. Review
identifies a proposal requiring judgment. Ordinary compilation/checking leaves
sources untouched; `check --fix` explicitly applies automatic proposals. An edit consumer must check the original text and context again
before applying a proposal; a successful replacement does not establish the
correctness of the rest of the program.

Hints cover common Pascal type names, keyword capitalization, `elseif`, `=` in
initializers and `!=`. Console I/O and `exit` hints describe the supported API or
statement without fabricating edits that require unknown imports or values.

## Apply automatic corrections

Use an explicit project or solution and the usual configuration/restore options:

```sh
fpas check Example.fpasproj --fix --dry-run --diagnostics json
fpas check Example.fpasproj --fix --diagnostics json
```

`--fix --dry-run` previews one batch without source writes. `--dry-run` requires
`--fix`. `--fix` applies only automatic proposals from the captured analysis;
review proposals remain visible and untouched. No proposal is guessed from a
diagnostic message. Each invocation prepares a fresh batch; previews are not
persisted edit plans. New diagnostics exposed by a correction can need another
invocation or a manual change.

Only evaluated regular `.fpas` inputs are eligible, including linked sources
explicitly owned by the project. Reparse points and read-only files are refused.
Duplicate edits from shared source owners are applied once. Original UTF-8 BOM,
line endings, unrelated text, Unix permission bits and Windows access rules are
preserved, including during rollback. Unchanged files retain their timestamps.
Corrections do not format the source.

Missing/blocked project inputs, the diagnostic error limit, invalid edit ranges,
wrong original text, conflicting/overlapping edits and changed context inputs
prevent publication. The batch is bounded to 1,000 proposed edits and 64 MiB of
original/replacement bytes. JSON is bounded to 4 MiB; the preview is checked
against that bound before any write.

All replacements and rollback copies are staged beside their sources. Context
and original bytes are reverified before per-file replacement. A failed batch
attempts rollback without overwriting another writer's changes. If recovery
cannot restore every file, the failure lists written paths and retained recovery
files. Replacement is atomic per file; other applications can still change files
during a batch.

After application, the command reloads project inputs and dependency metadata,
then analyzes and verifies them again within the same timeout. Remaining errors
return exit 1 even when edits were applied; applied corrections stay on disk.
Reload, timeout or cancellation failures also retain explicit publication status.
Preparation can restore/build dependencies as with ordinary check; the selected
root's own outputs are preserved under the existing check contract.

JSON adds `fixes` with mode/state, original inputSnapshotId, proposals, file
hashes, unique edit counts, written flags, rechecked, review counts and optional recovery
paths. Top-level diagnostics/snapshot describe the final check after application,
or the original check during preview. An interrupted/failed reload reports no
stale pre-edit diagnostics; rechecked remains false. Human diagnostics, edit summaries and
previews go to stderr. JSON owns stdout, including failures. Exit codes remain
0 for a successful final check/help, 1 for remaining errors/setup/publication or
deadline failures, 2 for arguments and 130 for cancellation.

## Recovery and limits

The parser continues at safe statement boundaries and checks other source files.
The binder isolates routine bodies, record methods and source units, so a failed
routine does not contaminate another routine's flow state. It abandons the rest
of a failed local body to avoid dependent name/type errors. Invalid declarations,
imports or lexical structure can stop their owning file or compilation phase.
Metadata/emission checks are reached only after the preceding phase succeeds.

At most 50 errors are returned; the final `FP0003` diagnostic reports when that
limit prevents further analysis. Any error prevents output publication and a
successful semantic snapshot. Existing outputs remain intact. Correct the
reported problems and check again to reach later phases.

| Exit code | Meaning |
| --- | --- |
| 0 | Successful compilation or help |
| 1 | Compilation, input, or output failure |
| 2 | Invalid command-line arguments |

## Codes

| Codes | Meaning |
| --- | --- |
| FP0001 / FP0002 | Invalid CLI arguments / input-output failure |
| FP0003 | Diagnostic error limit reached; further errors were not analyzed |
| FP1001–FP1004 | Invalid character, string, separator, or unsupported numeric form |
| FP2001–FP2005 | Syntax/terminator error, missing type annotation, unsupported form, source/output-kind mismatch, or duplicate container |
| FP3001–FP3008 | Name, type, overload, binding, unsupported type, unused result, missing return, or unreachable statement |
| FP3009 / FP3010 | Non-writable assignment target / loop exit outside a loop |
| FP3011 / FP3012 | Private routine/member/constructor access / importing a program |
| FP3013 / FP3015–FP3017 | Nullable safety / property accessors, disposal, or exception-flow errors |
| FP3018 / FP3019 | Generic constraint failure / unsupported or unresolved callable signature |
| FP3021 / FP3022 | Invalid record declaration or visibility / invalid record construction or update |
| FP3023–FP3025 | Invalid union declaration, pattern, or coverage/unreachable case arm |
| FP3026 | Invalid/unsupported source generic declaration or explicit type-argument arity |
| FP3027 / FP3028 | Invalid contract / parameter modes, defaults, storage, or out flow |
| FP3029 / FP3030 | Invalid record interface implementation / async or await reference-lifetime error |
| FP3031 | Invalid alias, enum, distinct declaration, subrange bounds, or domain construction |
| FP3032 | Invalid global name or public global exposing private types |
| FP3033 | Nested routine environment created before captured bindings initialize |
| FP3034 | Invalid or ambiguous event selection/access, or event used as a delegate value |
| FP3035 | Invalid attribute class, target, duplicate, constant value, named member, or structural metadata |
| FP6001 | Documentation warning; severity follows the configured warning policy |
| FP4001 / FP4002 | Invalid/missing reference metadata / ambiguous reference identity |
| FP4003 | Invalid strong-name signing key or signing mode |
| FP5001 | Invalid or unsupported SDK project configuration |
| FP5002 | Build-adapter input/output or cleanup failure |
| FP5003 | Compiler process failure or missing requested assembly |
| FP5004 | Compilation canceled or timed out through the build adapter |
| FP5011 | Publish certificate signing or pinned-signature verification failure |
| FP7001 | Formatter preservation check failed; the source is not rewritten |

## Warning policy

XML documentation diagnostics use `FP6001`. `--warn-as-error` promotes warnings;
`--warnings-as-errors` promotes selected codes, `--warnings-not-as-errors` excludes
selected codes from the global promotion, and `--no-warn` suppresses selected
codes. Code lists accept commas or semicolons and either FP6001 or 6001 notation.
Promoted warnings fail compilation
and prevent publication of new outputs.

The project SDK reports diagnostics through MSBuild with start/end coordinates
and warning severity. Help, explanations and related locations are logged as
messages, so they do not create extra error-list entries. Structured payloads
remain in compiler JSON and cached warning reports. IDE presentation is subject
to the IDE's MSBuild support.
See the [SDK configuration](../../build/msbuild.md) for project warning properties
and the [compiler CLI](../program-structure/cli.md) for direct invocation.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
