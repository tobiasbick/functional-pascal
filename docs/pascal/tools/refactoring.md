# Checked source refactoring

`fpas refactor` prepares exact source edits as a schema-2 JSON report. Preparation
does not change source. Save the report with `--out`, then use `apply --plan` to
revalidate and publish it. Commands are non-interactive.

```sh
fpas refactor rename --symbol App.Math.Clamp --name Bound --context app.fpasprj --out rename.plan.json
fpas refactor extract-variable --range src/main.fpas:12:18:12:25 --context app.fpasprj --out extract.plan.json
fpas refactor organize-imports src/main.fpas --context app.fpasprj --out imports.plan.json
fpas refactor add-parameter --symbol App.Math.Clamp --context app.fpasprj --name Offset --type integer --mode value --argument '0' --out parameter.plan.json
fpas refactor apply --plan rename.plan.json
```

## Selection and eligibility

Rename and add-parameter accept one `file.fpas:line:column`, `--symbol <exact-name>`, or
`--target-ref <file>` from a checked [source query](source-queries.md). Names
require an explicit `.fpasprj` or `.fpasworkspace` context. Positions can discover
their owning project, or use a loose source if no manifest owns it. Coordinates
use one-based lines and Unicode-scalar columns. Extraction ranges are half-open;
Windows drive colons remain part of the path.

`--std-lib <dir>` selects a source standard library during preparation. The plan
records that choice, its complete input inventory and the physical context root.
Apply uses those recorded identities independently of its working directory.

Rename rejects invalid identifiers, keywords, binding conflicts, compilation-unit
names, generated APIs, standard-library sources and edits outside the selected
root. Select a containing workspace to include a shared library and its consumers.
Consumers outside that context are not discovered automatically.

Extraction uses a deterministic, collision-free local name recorded in
`result.plan.recipe.name`. It rejects selections that would change evaluation
order, frequency or binding. Import organization preserves aliases, comments and
observable initialization order. Comments within a `uses` clause can prevent
organization; already organized imports produce no change.

Every selected project and consumer is checked before and after the proposed
edits in memory. Errors in an unedited input also block the transformation.
Exact untouched UTF-8 bytes and line endings are preserved.

## Add parameter

`add-parameter` appends one parameter and one explicit argument at every supported
call. All four options are required: `--name`, `--type`, `--mode value|var` and
`--argument`. Type and argument must each be a complete existing FPAS fragment;
quote them for the calling shell. The type resolves in the declaration's scope.
The argument resolves separately in each caller. For `var`, supply an unmarked
writable designator; the generator adds `var` and any named-argument label.

Supported declarations are named non-generic functions and procedures, including
nested routines outside generic bodies and exported routines within the selected
consumer context. Ordinary expression/statement calls, recursion and import
aliases are supported. A non-generic record default is edited once at its physical
declaration, regardless of the number of constructions.

Generic declarations or affected generic bodies/defaults, methods, constructors,
receiver-based and spawned calls are unsupported. Callable-value/callback uses
and unresolved invocations block the whole plan. Method-based implicit resource
release operations are excluded with their method declaration. There is no partial
propagation, default argument, automatic import or per-caller expression policy.
Concrete applied generic types remain valid parameter type expressions.

The checker rejects duplicate or invisible names/types, invalid `var` storage and
overlapping storage roots. It compares unchanged bindings and call targets across
all edits and independently checks new arguments before introducing the requested
parameter name. An argument cannot refer to the modified routine, including through
recursive calls. Existing positional/named argument order and comments remain
unchanged. The new argument evaluates last, once when reached; its effects, failure
or `try` propagation may prevent the call. Only selected consumers are validated.

The complete use inventory has no presentation limit. Preparation rejects more
than 65,536 affected physical calls or 8,192 blocking sites; each retained binding
comparison is limited to 32 MiB. These limits never produce a partial plan. Normal
plan byte limits and retry rules apply; repeat application adds no second parameter.

## Reports and saved plans

The report uses the same envelope as source queries, with `result.kind: refactor`.
`result.status` distinguishes `prepared`, `no_change`, `unsupported`, `incomplete`,
`applied` and `already_applied`. Blockers retain semantic reasons and original
compiler diagnostics. An eligible no-change result includes an empty checked plan.

Version-1 plans contain the normalized recipe, shared analysis identity, complete
before/expected-after inventories, ordered byte edits with exact old/new text and
a deterministic content digest. The digest detects altered content; it is not
authority to write files. Apply reconstructs the context and regenerates the edits.

Without `--out`, stdout is the complete preview artifact. With `--out`, the same
UTF-8 envelope is also published atomically. The destination's parent must exist.
An identical existing artifact is accepted; a different file is preserved and
reported as an error. Output cannot alias an analysis/reference input or change
recorded source membership. Artifact-output failures do not modify source.

The public report and plan-file ceiling is 8 MiB; source replacement preparation
is bounded by 128 MiB. Exact edit text is never truncated into an applicable plan.

## Apply, retry and publication failures

Apply compares complete inventories before resolving an old declaration:

- Matching before state regenerates and validates the exact recipe and edits.
- Matching only the complete expected-after state returns `already_applied`
  without writes. This describes saved state, not authenticated execution history.
- Any other state returns `stale_plan`, including partial application or changes
  to an unedited source, manifest, path binding or glob membership.

An empty plan still requires recipe validation and returns `no_change`. Repeating
an extraction or import removal therefore does not insert or remove more text.

`--timeout <seconds>` defaults to 30 seconds. Analysis runs in an isolated process.
The parent stages every replacement, preserves permissions and verifies all inputs
before the first source replacement. Once replacement begins, it finishes
publication or ordinary recovery independently of that preparation deadline.

`result.publication` reports each path as `unchanged`, `replaced`, `restored` or
`unresolved`, plus the publication error and recovery errors. Recovery preserves
concurrent newer text instead of overwriting it with an old snapshot. These are
ordinary failure guarantees, not crash-atomic multi-file transactions. A final
stdout failure is reported on stderr; inspect saved files before retrying.

Exit 0 means a successful prepared/no-change/applied/already-applied result. Exit 1
means an invalid request, stale or malformed plan, unsupported transformation or
artifact-output failure. Incomplete analysis, deadlines, worker failure and source
publication failure exit 3. `--help` describes each command without starting analysis.

## Editor preparation and application

Standard LSP rename and extract/import actions use the same checked core recipes
as the CLI. The `fpas/refactor` extension advertises version 1 and accepts
`{"version":1,"phase":"prepare","context":uri,"recipe":recipe}`. Context is
optional for positional recipes. Editor selectors use `uri` and a zero-based
UTF-16 `position`; extraction uses `uri` and a half-open LSP `range`, while
import organization uses `uri`. Symbol and reference selectors retain the core
`name` and `target_ref` fields.

The parameter recipe uses `action: "add-parameter"`, `selector`, `name`, `type`,
`mode` and `argument`, with the same rules as CLI. In VS Code,
**Functional Pascal: Add Parameter** collects missing values and shows a read-only
preview before application. Automation can invoke `functionalPascal.addParameter`
with `uri`, zero-based UTF-16 `position`, optional `context`, and all four recipe
values to apply without prompts. Alternatively, use `symbol` with a manifest
`context`. `previewOnly: true` returns the checked result without applying it.
The command returns the semantic result and `previewUri`; successful application
has status `applied`. It reuses the active language client and never saves buffers
or launches a CLI subprocess.

Replies contain `status`, `context`, `snapshot`, `handle`, `edit`, `blockers`,
`diagnostics`, `limitations` and `coverage`. Revalidation accepts
`{"version":1,"phase":"revalidate","handle":handle}`. Handles belong to the
current server session and expire when evicted from its bounded cache (128 entries,
32 MiB of plans). Requests, responses and individual plans have an 8 MiB ceiling.
They are not CLI plan files. Unknown or stale handles require fresh preparation.

Extract/import actions carry only a `fpas.applyRefactor` command with one handle.
Execution revalidates the whole read set and then asks the client to apply a
versioned `WorkspaceEdit`. Missing apply/documentChanges capabilities omit the
actions; a refused edit reports `editor_rejected`. Open buffers remain unsaved,
including edits across several files, and normal editor undo remains available.

Any participating buffer version change invalidates a prepared handle, even if
its bytes are unchanged. Closed files, manifests, aliases and glob membership are
checked before edit delivery. Standard LSP edits cannot atomically guard unedited
dependencies or closed files against changes after delivery and before the client
applies them; the adapter reports stale state it can observe before delivery.
