# Source-tool contract, version 2

This is the concrete contract for Q85–Q91. The core owns typed selectors,
snapshots, coverage, declaration facts and edit recipes. Adapters own wire
positions and presentation. JSON objects reject unknown selector/recipe fields;
lists have deterministic order. A missing value is `null`, not an omitted key.

## Canonical identities

SHA-256 digests are lowercase hexadecimal. Canonical encoding is a sequence of
byte strings, each preceded by its unsigned 64-bit little-endian byte length.
Strings are UTF-8 without Unicode normalization. Integers in identity records
are decimal ASCII without leading zeros. The first string is the domain and
version. Lists begin with their decimal element count. Optional values use an
empty list for absence or a one-element list for presence. This encoding is
independent of JSON field order, escaping, local editor revisions and adapters.

Framing vector: the strings `a`, `bc` encode as
`01000000000000006102000000000000006263`. Empty bytes hash to
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
the raw UTF-8 bytes `abc` hash to
`ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`.

Identity paths are absolute and lossless UTF-8. Canonical physical paths are
resolved by the filesystem; lexical paths are made absolute without removing
symlink-sensitive `..` components. Windows verbatim drive and UNC prefixes are
converted to ordinary absolute paths and separators to `/`; case is preserved
from canonicalization, never folded on case-sensitive volumes. On Unix a literal
backslash stays a backslash. Non-UTF-8 paths return `unsupported_path_identity`.
Display paths are separate and may be relative to the invocation directory.

Input inventories contain sorted physical files (`path`, `sha256`, `bytes`),
sorted lexical/physical aliases, sorted selected physical identities, and sorted
glob records (`root`, `pattern`, ordered exclusions, sorted lexical members).
Files include exact manifest/source bytes; BOM and line endings count. Membership
is verified by repeating discovery, including originally empty globs. A missing
or retargeted input is stale. Unread admitted files prevent a complete inventory.

The snapshot domain is `fpas-source-snapshot-v1`. Its fields are core build ID,
context kind, absolute context input, physical root, optional standard-library
root, policy, project identities and the complete inventory digest. Result limits and requested operation are
excluded. The same saved bytes and policy give the same identity in CLI and LSP.
An editor overlay replaces that file's byte hash; editor versions are additional
local guards. Disk changes to overlaid source bytes do not replace editor bytes.

The build domain is `fpas-analysis-core-v1`. Build inputs are `Cargo.toml`,
`Cargo.lock`, the core's recursive local production/build dependency manifests,
all Rust sources and build scripts in those packages, plus sorted Cargo target
configuration and enabled core features. Each file contributes its workspace-
relative path and exact bytes. Test modules embedded in source directories are
conservatively included; separate tests/examples and adapter sources are not.
The whole lockfile is deliberately conservative. A failed identity build fails
compilation instead of issuing a constant fallback. Compiler artifact identity
remains owned by `fpas-build` and is not replaced by this fingerprint.

## Requests and reports

Selectors are tagged by `kind`: `position` has `path` and `offset` (UTF-8 byte
offset in the core), `symbol` has `name`, and `reference` has `target_ref`.
CLI positions convert one-based Unicode-scalar line/column; LSP positions convert
zero-based UTF-16 line/character. Neither conversion changes semantic identity.
`find` uses `{"kind":"search","text":"..."}`. Context is a manifest or a source path
for implicit positional discovery. Symbol/find requests require a manifest.

Every CLI report has these fields, in this presentation order:

```json
{
  "schema_version": 2,
  "operation": "definition",
  "outcome": "not_found",
  "exit_code": 1,
  "request": {"selector": null, "arguments": {}, "options": {}},
  "context": null,
  "snapshot": null,
  "target": null,
  "candidates": [],
  "result": {"kind": "definition", "locations": []},
  "diagnostics": [],
  "coverage": {"analysis_complete": true, "reasons": [], "collections": {}, "omissions": []},
  "limits": {"timeout_ms": 30000, "input_files": 4096, "file_bytes": 8388608, "input_bytes": 67108864, "json_bytes": 8388608, "preparation_bytes": 134217728, "excerpt_bytes": 32768, "supporting_types": 12}
}
```

`operation` is null for an unrecognized command. Invalid arguments yield `error`,
exit 1 and diagnostic code `invalid_request`. Internal/worker failure yields
`error`, exit 3 and `worker_failure`. Cancellation/deadline and incomplete analysis
yield `incomplete`, exit 3. Complete absence is `not_found`, exit 1. Successful
empty discovery or a resolved declaration with zero uses is `ok`, exit 0.

Each collection reports `returned`, `known_total` (integer or null), `complete`.
`result_limit` clips collections, never establishes uniqueness, and does not by
itself invalidate verified references. `analysis_errors`, `analysis_unavailable`,
`source_limit`, `source_changed`, `query_cancelled` and `execution_timeout` prevent
references. Diagnostics preserve FP code, message, severity, hints and attributed
source ranges. Tool diagnostics use a stable `code`, `message`, optional location.

Payloads: `find` has `symbols`; `definition`/`refs` have `locations`; `type` has
`type` or null and `unavailable_reason`; `inspect` has ordered `sections` and
`omissions`; calls have separate `direct_calls`, `callable_values`,
`callback_arguments`, `dynamic_calls`, per-collection counts and
`direct_edges_complete`/`runtime_targets_complete`. An unresolved site never
becomes a fabricated edge. Inspection sections have exact continuous excerpts,
full/omitted byte counts and explicit omission reasons. Generated stubs use
`intrinsic_implementation` as their unavailable body reason. Source-less builtins
use `no_source` for references. Native call targets have compiler identities and
null source locations. `body_unavailable` limits a callee query without making
an otherwise verified declaration reference stale.

Ambiguity uses `error`, exit 1, null target, `ambiguous_target` diagnostic and
candidate declarations. Incomplete discovery remains `incomplete` even if one
candidate survives. A well-formed mismatching reference yields `stale_target`;
malformed references yield `invalid_request`. References contain version 1,
context input, library selection, policy, snapshot/build IDs, project owner,
kind, qualified name, canonical file, defining name byte span and file digest.

## Refactoring recipes and plans

Recipes are tagged by `action`: `rename` carries selector/name;
`extract-variable` carries path/start/end/name; `organize-imports` carries path;
`add-parameter` carries selector/name/type/mode/argument. Type and argument are
complete existing-language fragments. Modes are `value` and `var`.

A version-1 plan contains `plan_version`, `analysis_identity`, `context`,
`recipe`, `before`, `after`, `edits`, `digest`. The two inventories cover all
inputs, not only edited files. Edits are sorted by identity path and byte span,
with exact `old_text` and `new_text`; overlaps are invalid. The digest uses domain
`fpas-refactor-plan-v1` and the same framing over these fields in the stated
order, excluding the digest itself. The version is a decimal integer; context
fields are input, kind, root, optional library, projects and policy. Recipes frame
the action then their fields in the order above; selectors frame kind then their
fields. A reference frames version, context, snapshot/build IDs, optional owner,
declaration kind (the stable source-symbol variant spelling), qualified name,
file, start, length and file hash. Before/after values contribute their complete
inventory digests. Edits frame path, start, length, old text and new text.
It is corruption detection, not authority.
Decoding is bounded by 8 MiB. Original/replacement preparation is bounded by
128 MiB. Oversized edits fail with `plan_limit` before any source publication.

Preview returns `result.kind: refactor`, `status: prepared`, a plan and no writes.
An eligible empty transformation returns `no_change` after validation.
Unsupported syntax or use yields `unsupported_refactoring` with exact blockers;
missing analysis yields incomplete, never no-change. An optional plan output
cannot alias an input, edit target, reference input or manifest.

Apply first validates structure/digest/context. It compares the complete current
inventory before resolving an old selector. Matching before regenerates and
compares exact checked edits. Matching only after returns `already_applied` with
no writes; matching neither returns `stale_plan`. Equal before/after still requires
recipe validation and returns `no_change`. This prevents duplicate extraction or
parameter insertion. Paths resolve from the recorded base, not the new cwd.

Publication results contain `status`, `paths`, `publication_error` and
`recovery_errors`. Each path has `path`, `state` (`unchanged`, `replaced`,
`restored`, `unresolved`) and optional `error`. All replacements stage before
publication. The preparation deadline ends before the first replacement; the
coordinator then finishes publication or recovery. Recovery preserves concurrent
newer text and reports unresolved files. A final stdout failure uses stderr even
if publication succeeded. No crash-atomic multi-file transaction is promised.

## LSP extensions and worked lifecycle cases

The server advertises `experimental.fpasSourceQuery: {"version": 1}` and
`experimental.fpasRefactor: {"version": 1}`. `fpas/sourceQuery` params are
`version`, `operation`, `context`, `selector`, `limit`, `policy`. Search text is
carried by `selector: {"kind":"search","text":"Clamp"}`.
`fpas/refactor` params are `version`, `phase` (`prepare`/`revalidate`) and either
`recipe`/`context` or `handle`. Unknown versions fail as invalid params. Normal
JSON-RPC cancellation is honored. Replies retain typed semantic status, coverage,
blockers and facts; refactoring replies additionally project versioned edits and
a session-local handle. Handles never act as portable references or plan files.

For saved `Clamp` with one direct caller, CLI and rich LSP queries return the same
declaration byte span, snapshot ID and call edge. LSP ranges alone use UTF-16.
Adding a supplementary Unicode character before the caller changes its UTF-8
offset by four, scalar column by one and UTF-16 character by two.

For a dirty caller, the LSP snapshot includes the dirty bytes and its document
version. A reference from that snapshot is stale in disk-only CLI until those
exact bytes are saved. Revalidating an edit after any participating dirty buffer
changes returns stale, including changes in unedited dependencies. Versions on
edited open documents go into `documentChanges`; closed-file hashes are checked
before delivery. Standard edits cannot atomically guard subsequent changes to
unedited dependencies or closed files.

Extract/import actions carry only a command and snapshot-bound recipe handle.
Invocation revalidates the full read set before `workspace/applyEdit`. Missing
apply capability disables that action; client rejection reports failure. The
add-parameter command uses the same prepare/revalidate path, leaves buffers dirty
and preserves undo. Inspect uses the rich query sections including omissions.
